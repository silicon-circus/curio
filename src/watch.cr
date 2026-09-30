require "./config"
require "./store"

module Curio
  # watch/ — A COMMAND, NOT A DAEMON.
  #
  # Tom: "A .glb file I dropped in and was about to adjust the name of disappeared instantly. My
  # fault, should have adjusted the name first, but I think it would be better if watch wasn't a
  # constant background task and required a command on my part to trigger, with output of what it
  # plans to do and asking for confirmation."
  #
  # Right, and the two-second timer was never the safety it looked like. A folder you are still
  # arranging is indistinguishable from a folder you have finished arranging, so a background task
  # has to guess, and it guessed while his hand was still on the mouse. Making it a command replaces
  # the guess with the only thing that actually knows: him saying go.
  #
  # AND IT PRINTS THE PLAN FIRST. Not a progress log after the fact — a list of what it is about to
  # do, before it does any of it. That is what turns "it disappeared" into "no, not that one".
  #
  # THE PATH IS PART OF THE NAME. The old version took File.basename, which threw the directory away:
  # 370 files of a Kenney kit, each buried three folders deep, all landed in names/ as barrel.obj,
  # structure.obj, colormap.png — flat, unrelated, and four of them colliding with each other. So:
  # a top-level DIRECTORY in watch/ is a collection and keeps its shape; a top-level FILE is one
  # asset. Those are the only two cases, and neither of them invents a name.
  module Filing
    extend self

    enum Kind
      Collection
      Single
      Held
    end

    record Item,
      kind : Kind,
      source : String,          # absolute path in watch/
      name : String,            # what it will be called in names/
      files : Int32,
      bytes : Int64,
      why : String = ""

    # Look, decide, and report — WITHOUT touching anything.
    def plan : Array(Item)
      items = [] of Item
      return items unless Dir.exists?(Config.watch)

      Dir.each_child(Config.watch) do |child|
        next if child.starts_with?('.')
        path = File.join(Config.watch, child)
        name = Store.safe_name(child)

        if name.empty?
          items << Item.new(Kind::Held, path, child, 0, 0_i64, "no usable name")
          next
        end
        dest = File.join(Config.names, name)

        if Dir.exists?(path)
          n, bytes = 0, 0_i64
          Store.walk(path) { |f, _| n += 1; bytes += File.info(f).size }
          if n == 0
            items << Item.new(Kind::Held, path, name, 0, 0_i64, "empty directory")
          elsif File.exists?(dest)
            items << Item.new(Kind::Held, path, name, n, bytes,
              "names/#{name}/ already exists — rename this, or unpublish that")
          else
            items << Item.new(Kind::Collection, path, name, n, bytes)
          end
        else
          size = File.info(path).size
          if File.exists?(dest)
            same = Store.sha256(dest) == Store.sha256(path)
            items << Item.new(Kind::Held, path, name, 1, size,
              same ? "names/#{name} is already served, same bytes — safe to delete"
                   : "names/#{name} is already served, DIFFERENT bytes")
          else
            items << Item.new(Kind::Single, path, name, 1, size)
          end
        end
      end
      items.sort_by! { |i| {i.kind.value, i.name } }
      items
    end

    def describe(items : Array(Item), io : IO = STDOUT)
      if items.empty?
        io.puts "watch/ is empty"
        return
      end
      items.each do |i|
        case i.kind
        when Kind::Collection
          io.puts "  COLLECTION  #{i.name}/"
          io.puts "              -> names/#{i.name}/   #{i.files} files, #{mb(i.bytes)}"
        when Kind::Single
          io.puts "  FILE        #{i.name}"
          io.puts "              -> names/#{i.name}   #{mb(i.bytes)}"
        when Kind::Held
          io.puts "  HELD        #{i.name}"
          io.puts "              #{i.why}"
        end
      end
      go   = items.reject { |i| i.kind == Kind::Held }
      held = items.count { |i| i.kind == Kind::Held }
      io.puts
      io.puts "#{go.size} to file (#{go.sum(&.files)} files, #{mb(go.sum(&.bytes))})" +
              (held > 0 ? ", #{held} held" : "")
    end

    def mb(b : Int64) : String
      b > 1_000_000 ? "#{(b / 1_000_000.0).round(1)} MB" : "#{(b / 1024.0).round(0).to_i} kB"
    end

    # Do exactly what the plan said, and nothing that was not in it — INCLUDING checking again.
    #
    # The plan tested for collisions and then apply did not, which meant the guarantee only held for
    # the instant the plan was printed. `cp` overwrites by default, so anything appearing in names/
    # between the plan and the keypress would have been silently clobbered — and the whole reason
    # this became a command was to put a human pause exactly there. A promise that only holds while
    # nobody is looking is not the promise that was made.
    #
    # So every destination is re-tested at the moment of writing, and a race loses safely: the file
    # stays in watch/, is reported, and nothing is overwritten.
    def apply(items : Array(Item), io : IO = STDOUT) : {filed: Int32, files: Int32, skipped: Int32}
      filed = files = skipped = 0
      items.each do |i|
        next if i.kind == Kind::Held
        dest = File.join(Config.names, i.name)
        if File.exists?(dest) || Dir.exists?(dest)
          io.puts "  SKIPPED #{i.name} — appeared in names/ since the plan was made"
          skipped += 1
          next
        end
        if i.kind == Kind::Collection
          Dir.mkdir_p(dest)
          Store.walk(i.source) do |f, rel|
            Store.ingest(f)                                   # dedup, per file, as always
            target = File.join(dest, rel)
            next if File.exists?(target)                       # never write over anything, ever
            Dir.mkdir_p(File.dirname(target))
            Store.copy_cow(f, target)
            files += 1
          end
          rm_r(i.source)
        else
          Store.ingest(i.source)
          Store.copy_cow(i.source, dest)
          File.delete(i.source)
          files += 1
        end
        filed += 1
        io.puts "  filed #{i.name}#{i.kind == Kind::Collection ? "/" : ""}"
      end
      {filed: filed, files: files, skipped: skipped}
    end

    def rm_r(path : String)
      Dir.each_child(path) do |c|
        p = File.join(path, c)
        Dir.exists?(p) ? rm_r(p) : File.delete(p)
      end
      Dir.delete(path)
    end
  end
end
