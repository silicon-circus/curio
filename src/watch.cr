require "./config"
require "./store"

module Archive
  # watch/ — DROP IT AND FORGET IT.
  #
  # Tom's workflow, in his words: "AI generates an asset, drops into intake/ for my approval. I need
  # only to move it to watch/ with my chosen name and that's basically it."
  #
  # So watch/ is the yes. The name on the file is the name it will be served under, and there is
  # nothing else to decide — hash, store, publish, gone.
  #
  # THE ONE THING IT WILL NOT DO IS OVERWRITE. Tom: "watch should also make sure a file by the same
  # name doesn't already exist and flag that and not ingest it." A collision is left exactly where it
  # is, in watch/, and reported. Not moved, not renamed, not merged, not quietly dropped even when
  # the bytes are identical — because "I moved it to watch/ and it vanished" is indistinguishable
  # from "I moved it to watch/ and it worked", and those are very different things to have happened.
  # You rename it or you clear the old name; either way the decision stays yours.
  class Watcher
    # A file is only filed once its size has stopped changing. Copying a 60 MB glb into the folder
    # takes a moment, and hashing it halfway through would store a truncated object under a hash that
    # is real, permanent, and wrong.
    SETTLE = 2

    def initialize
      @seen = {} of String => {Int64, Int64}     # path => {size, when it stopped growing}
      @blocked = {} of String => String          # path => why it is still sitting there
      @log = [] of String
    end

    getter log
    getter blocked

    def note(msg : String)
      @log << "#{Time.local.to_s("%H:%M:%S")}  #{msg}"
      @log.shift if @log.size > 200
      Log.info { msg }
    end

    def run
      spawn do
        loop do
          begin
            tick
          rescue ex
            note "watch error: #{ex.message}"
          end
          sleep 2.seconds
        end
      end
    end

    def tick
      now = Time.utc.to_unix
      present = Set(String).new
      Store.walk(Config.watch) do |path, rel|
        present << rel
        size = File.info(path).size
        prev = @seen[rel]?
        if prev.nil? || prev[0] != size
          @seen[rel] = {size, now}                 # still growing; start the clock again
          @blocked.delete(rel)                     # it changed, so give it another go
          next
        end
        next if now - prev[1] < SETTLE
        next if @blocked.has_key?(rel)             # already reported; say it once, not every 2 s
        file(path, rel)
      end
      # anything that left the folder stops being our business
      @seen.reject! { |k, _| !present.includes?(k) }
      @blocked.reject! { |k, _| !present.includes?(k) }
    end

    def block(rel : String, why : String)
      @blocked[rel] = why
      note "HELD  #{rel} — #{why}"
    end

    def file(path : String, rel : String)
      name = Store.safe_name(File.basename(rel))
      return block(rel, "no usable name") if name.empty?

      existing = File.join(Config.names, name)
      if File.exists?(existing)
        same = Store.sha256(existing) == Store.sha256(path)
        block(rel, same ? "#{name} is already served, and these are the same bytes — safe to delete"
                        : "#{name} is already served, and these are DIFFERENT bytes")
        return
      end

      hash = Store.ingest(path)
      unless Store.publish(hash, File.extname(path).downcase, name)
        block(rel, "could not publish")
        return
      end
      File.delete(path)
      @seen.delete(rel)
      note "filed #{name}  #{hash[0, 12]}"
    end
  end
end
