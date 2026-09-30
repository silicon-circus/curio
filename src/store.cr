require "digest/sha256"
require "./config"

module Curio
  # The store: hashing, linking, and the three one-way doors between the folders.
  module Store
    extend self

    record Entry, name : String, hash : String, size : Int64, mtime : Int64 do
      def ext : String
        e = File.extname(name)
        e.empty? ? "" : e.downcase
      end
    end

    def sha256(path : String) : String
      digest = Digest::SHA256.new
      File.open(path) do |f|
        buf = Bytes.new(1024 * 1024)
        while (n = f.read(buf)) > 0
          digest.update(buf[0, n])
        end
      end
      digest.final.hexstring
    end

    def object_path(hash : String, ext : String) : String
      File.join(Config.objects, "#{hash}#{ext}")
    end

    # Put bytes in the store. Returns the hash. If the store already has them, the file we were given
    # is simply linked to what is already there — which is the whole reason the store is
    # content-addressed: the second copy of anything costs nothing and is impossible to miss.
    # NEVER A HARDLINK INTO THE STORE, and this cost me an object to learn.
    #
    # This used to be File.link. sync calls it on files that go on LIVING in names/ — so the moment
    # an edit was filed away, the new object and the name shared an inode again, which is precisely
    # the hazard reflink copies were introduced to remove. Twenty minutes later I restored a file
    # with `cp` (which truncates the destination in place), and the write went straight through the
    # shared inode into the object. `just verify` found it: an object whose bytes hashed to something
    # other than its own filename.
    #   So: copy, never link. It is free on a reflink filesystem, and on one without it is the price
    # of the store meaning anything at all.
    def ingest(path : String) : String
      hash = sha256(path)
      ext  = File.extname(path).downcase
      dest = object_path(hash, ext)
      copy_cow(path, dest) unless File.exists?(dest)
      hash
    end

    # A COPY THAT COSTS NOTHING. `cp --reflink=auto` clones the extents where the filesystem can and
    # falls back to a real copy where it cannot, so this is correct everywhere and free on XFS.
    # Crystal has no binding for the clone ioctl and coreutils has had one that works since 2009.
    def copy_cow(src : String, dest : String) : Bool
      Process.run("cp", ["--reflink=auto", "--preserve=timestamps", src, dest],
                  output: Process::Redirect::Close, error: Process::Redirect::Close).success?
    end

    # Give an object a name. Publishing is making a WRITABLE copy of the object under that name —
    # not a hardlink, because a name is a thing Tom edits and an object is a thing that must never
    # change underneath one.
    def publish(hash : String, ext : String, name : String) : Bool
      src  = object_path(hash, ext)
      return false unless File.exists?(src)
      dest = File.join(Config.names, name)
      return false if File.exists?(dest)
      Dir.mkdir_p(File.dirname(dest))
      copy_cow(src, dest)
    end

    # THE ONE CHORE, ANSWERED YES. Hash it, name it, and drop the intake link — the bytes never
    # move, so this cannot fail halfway and leave a file in two states.
    def keep(file : String, name : String) : {ok: Bool, hash: String, why: String}
      src = File.join(Config.intake, file)
      return {ok: false, hash: "", why: "no such file in intake"} unless File.file?(src)
      name = safe_name(name)
      return {ok: false, hash: "", why: "bad name"} if name.empty?
      if File.exists?(File.join(Config.names, name))
        return {ok: false, hash: "", why: "#{name} is already being served"}
      end
      hash = ingest(src)
      unless publish(hash, File.extname(src).downcase, name)
        return {ok: false, hash: hash, why: "could not publish"}
      end
      File.delete(src)
      {ok: true, hash: hash, why: ""}
    end

    # ...AND ANSWERED NO. Into trash/, not into oblivion. If the same filename is thrown away twice
    # the second one gets a suffix, because silently overwriting the first would be exactly the loss
    # trash/ exists to prevent.
    def discard(dir : String, file : String) : Bool
      src = File.join(dir, file)
      return false unless File.file?(src)
      dest = File.join(Config.trash, File.basename(file))
      n = 1
      while File.exists?(dest)
        ext = File.extname(file)
        dest = File.join(Config.trash, "#{File.basename(file, ext)}~#{n}#{ext}")
        n += 1
      end
      File.rename(src, dest)
      true
    end

    # Unpublishing removes a NAME. The object stays: something else may be pointing at those bytes,
    # and even if nothing is, an archive that deletes data when you rename a link is not an archive.
    def unpublish(name : String) : Bool
      discard(Config.names, name)
    end

    # A name is a URL, so it has to survive being one, and it has to stay inside serve/.
    def safe_name(name : String) : String
      n = name.strip.gsub(/\s+/, "-")
      return "" if n.empty? || n.includes?("..") || n.starts_with?("/")
      n.gsub(/[^A-Za-z0-9._\-]/, "")
    end

    # ── the index ─────────────────────────────────────────────────────────────
    # What is servable, and which object each name points at. Rebuilt by scanning serve/, but only
    # re-hashing files whose size or mtime moved — hashing 1.7 GB on every boot would make the
    # server take ten seconds to start, and it would learn nothing new 99% of the time.
    def load_index : Hash(String, Entry)
      rows = {} of String => Entry
      return rows unless File.exists?(Config.index_path)
      File.each_line(Config.index_path) do |line|
        p = line.split('\t')
        next unless p.size == 4
        rows[p[0]] = Entry.new(p[0], p[1], p[2].to_i64? || 0_i64, p[3].to_i64? || 0_i64)
      end
      rows
    end

    def save_index(idx : Hash(String, Entry))
      File.open(Config.index_path, "w") do |f|
        idx.keys.sort.each do |k|
          e = idx[k]
          f.puts "#{e.name}\t#{e.hash}\t#{e.size}\t#{e.mtime}"
        end
      end
    end

    # SYNC IS WHAT FILES YOUR EDITS AWAY. Every name in names/ is hashed (only when its size or
    # mtime moved — hashing 1.7 GB on every boot would cost ten seconds to learn nothing), and any
    # bytes the store has not seen are ingested. So touching up a picture in place does not destroy
    # what it replaced: the previous version is already an object, it keeps its hash, and it simply
    # stops having a name. That is version history, and it costs nothing to keep because objects are
    # never deleted.
    def sync : {scanned: Int32, hashed: Int32, ingested: Int32, dropped: Int32}
      old = load_index
      idx = {} of String => Entry
      scanned = hashed = ingested = 0
      objects = Dir.children(Config.objects).to_set

      walk(Config.names) do |path, rel|
        scanned += 1
        info = File.info(path)
        size, mtime = info.size, info.modification_time.to_unix
        prev = old[rel]?
        if prev && prev.size == size && prev.mtime == mtime
          idx[rel] = prev
          next
        end
        hashed += 1
        hash = sha256(path)
        ext  = File.extname(rel).downcase
        unless objects.includes?("#{hash}#{ext}")
          dest = object_path(hash, ext)
          copy_cow(path, dest) unless File.exists?(dest)     # copy, never link -- see ingest
          objects << "#{hash}#{ext}"
          ingested += 1
        end
        idx[rel] = Entry.new(rel, hash, size, mtime)
      end

      save_index(idx)
      {scanned: scanned, hashed: hashed, ingested: ingested, dropped: (old.keys - idx.keys).size}
    end

    def walk(root : String, &block : String, String -> Nil)
      return unless Dir.exists?(root)
      stack = [root]
      while dir = stack.pop?
        Dir.each_child(dir) do |c|
          next if c.starts_with?('.')
          p = File.join(dir, c)
          if Dir.exists?(p)
            stack << p
          elsif File.file?(p)
            block.call(p, Path[p].relative_to(root).to_s)
          end
        end
      end
    end

    def intake_list : Array(NamedTuple(file: String, size: Int64, mtime: Int64))
      list = [] of NamedTuple(file: String, size: Int64, mtime: Int64)
      walk(Config.intake) do |path, rel|
        info = File.info(path)
        list << {file: rel, size: info.size, mtime: info.modification_time.to_unix}
      end
      list.sort_by! { |e| -e[:mtime] }
      list
    end
  end
end
