module Curio
  # Single source of truth: read from shard.yml at compile time so it cannot drift
  # from the packaged version.
  VERSION = {{ read_file("#{__DIR__}/../shard.yml").split("version:")[1].split("\n")[0].strip }}

  # FOUR FOLDERS, AND EACH ONE ANSWERS A DIFFERENT QUESTION.
  #
  #   watch/    "file this for me" — drop something in with the name you want and it is stored and
  #             published without being asked. No decision, so no chore.
  #   intake/   "is this a keeper?"  — the only chore. Things land here; Tom looks at them and says
  #             keep (with a name) or bin. The watcher sends anything ambiguous here rather than
  #             guessing, which is what keeps the automatic path safe.
  #   objects/  "have I already got these exact bytes?" — content-addressed, for dedup. Nobody ever
  #             reads a filename in here; it is not a place you browse.
  #   names/    "what can be asked for by name?" — the working set, and the filename IS the URL.
  #             Open one in GIMP and touch it up; that is the point.
  #   trash/    "what did I throw away?" — because deleting is the one action that cannot be undone,
  #             and the point of an archive is not losing things.
  #
  # NAMES ARE REFLINK COPIES, NOT HARDLINKS, and that one word is the whole safety of the thing.
  # A hardlink is the same inode, so an in-place edit — GIMP overwriting, `magick foo.png foo.png`,
  # any tool that opens for writing rather than writing-and-renaming — reaches THROUGH the name and
  # rewrites the object. The store would then hold bytes that do not hash to their own filename, and
  # every other name pointing at that object would have silently changed with it. Measured: a
  # hardlinked pair, one in-place write, both files changed.
  #   A reflink copy is a separate inode sharing the same extents. Editing it diverges only the
  # blocks you touched — objects/ cannot be reached from names/ at all. On XFS with reflink=1 (and
  # btrfs, bcachefs, APFS, ReFS, OpenZFS 2.2+) that costs nothing: measured, a 200 MB clone allocated
  # 0 MB, and a one-byte edit to a 5 MB file cost 0.1 MB. On ext4, which has no reflink, `cp` falls
  # back to a real copy — still correct, just no longer free.
  #
  # So: objects/ is the archive and only sync writes to it; names/ is yours to edit.
  module Config
    extend self

    def data_root : String
      ENV["CURIO_DATA"]? || File.expand_path("../data", __DIR__)
    end

    def port : Int32
      (ENV["CURIO_PORT"]? || "26037").to_i
    end

    # LOOPBACK BY DEFAULT, because there is no password on any of this.
    #
    # Four POST routes mutate the store — keep, trash, unpublish, sync — and none of them asks who
    # is calling. That is exactly right for a tool serving one machine's browser, and exactly wrong
    # the moment the port is reachable from anywhere else: unpublishing a name is one unauthenticated
    # request. Kemal binds 0.0.0.0 out of the box, which is a default chosen for demos.
    #
    # So the open state is opt-in and has to be typed. Set CURIO_BIND=0.0.0.0 only behind something
    # that terminates the public side and forwards nothing but GET — see "Deploying" in the README.
    def bind : String
      ENV["CURIO_BIND"]? || "127.0.0.1"
    end

    def watch : String   ; File.join(data_root, "watch")   ; end
    def intake : String  ; File.join(data_root, "intake")  ; end
    def objects : String ; File.join(data_root, "objects") ; end
    def names : String   ; File.join(data_root, "names")   ; end
    def trash : String   ; File.join(data_root, "trash")   ; end
    # Derivatives — a webp at some width — are objects too, but they also need a name to be found by,
    # and it is a name nobody chose: it is derived from the request. So they get their own directory
    # of links rather than cluttering serve/ with sizes.
    def cache : String   ; File.join(data_root, "cache")   ; end
    # What the server knows about names/ without having to re-hash 1.7 GB on every boot.
    def index_path : String ; File.join(data_root, "index.tsv") ; end

    def ensure_dirs
      [data_root, watch, intake, objects, names, trash, cache].each { |d| Dir.mkdir_p(d) }
    end
  end
end
