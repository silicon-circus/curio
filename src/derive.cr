require "./config"
require "./store"

module Curio
  # DERIVATIVES ARE OBJECTS TOO.
  #
  # A resized picture is not a new asset, it is the same asset answered differently — so it goes in
  # the store like everything else, gets found by its hash like everything else, and is made exactly
  # once. The second request for the same size is a static file read.
  #
  # ImageMagick does the work. Crystal has no image library worth the dependency, and shelling out to
  # a tool that has been correct since 1990 is not the place to be inventive.
  module Derive
    extend self

    RASTER = {".png", ".jpg", ".jpeg", ".gif", ".webp", ".tif", ".tiff", ".bmp", ".avif"}

    # Formats that have already thrown information away. Re-encoding one of these is not compressing
    # a picture, it is compressing somebody else's compression artifacts — and the encoder cannot
    # tell those from detail, so it spends bits preserving them.
    LOSSY = {".jpg", ".jpeg", ".webp", ".avif", ".gif"}

    # DOWNWARD ONLY, and this table is the whole rule.
    #
    # A request for a format that is not on disk may be answered by RE-ENCODING a master that is —
    # but only in the direction that loses. png/jpg → webp, never the reverse: a .png conjured out
    # of .webp bytes would be a lossless-looking file that is nothing of the sort, and every name it
    # was served under would be a quiet lie about what the archive holds. So a master is always
    # available in its own format, and nothing here ever fabricates one.
    DERIVABLE = {".webp" => [".png", ".jpg", ".jpeg"]}

    def raster?(ext : String) : Bool
      RASTER.includes?(ext.downcase)
    end

    # The name asked for is not a file. Is it one we are allowed to make, and out of what?
    # Lossless sources are tried first: given both foo.png and foo.jpg, the png is the better parent
    # for a webp because it is the one that has not already lost anything.
    def source_for(path : String) : String?
      ext = File.extname(path)
      sources = DERIVABLE[ext.downcase]?
      return nil unless sources
      stem = path.rchop(ext)
      sources.each do |src_ext|
        candidate = stem + src_ext
        return candidate if File.file?(candidate)
      end
      nil
    end

    # QUALITY FOLLOWS THE SOURCE, because the same number does not mean the same thing.
    #
    # Encoding webp from a png is compressing a picture: 82 is plenty and the artifacts it adds are
    # the first ones the file has ever carried. Encoding webp from a jpg is a SECOND lossy pass over
    # bytes that are already dented, so the encoder needs more room to avoid compounding what it
    # inherited — hence 90. Same-format resizing keeps the old default of 88 exactly, so nothing
    # that already worked changes its output.
    def default_quality(src_ext : String, target : String) : Int32
      return 88 if src_ext == target
      LOSSY.includes?(src_ext) ? 90 : 82
    end

    # THE CACHE KEY CARRIES THE FILE'S STATE, NOT JUST ITS NAME.
    #
    # Keyed on the name alone — which is how I first wrote it — a rendition outlives the master it
    # was made from: touch up boardwalk.cattacula.night.real.png and the site keeps being handed the
    # 320-wide version of the picture you just replaced, for ever, with nothing to tell you. So the
    # key includes size and mtime. Editing anything changes at least one of them, the old key is
    # simply never asked for again, and the next request renders fresh. A stat, not a hash: this
    # happens on every request and hashing a 3 MB master to serve a 14 kB thumbnail is not a trade.
    #
    # The extension is the TARGET, not the source, so foo.png answering /a/foo.webp caches as a webp
    # and gets served with the right content type by virtue of its own name.
    def cache_name(src : String, target : String, width : Int32?, quality : Int32) : String
      info = File.info(src)
      parts = [File.basename(src, File.extname(src)),
               "s#{info.size}", "m#{info.modification_time.to_unix}"]
      parts << "w#{width}" if width
      parts << "q#{quality}"
      "#{parts.join('.')}#{target}"
    end

    # Make `src` answer as `target`, optionally at `width`. Returns the path to serve, or nil if the
    # conversion failed. Both a format change and a resize come through here, and either may be the
    # only thing asked for.
    def render(src : String, target : String, width : Int32?, quality : Int32?) : String?
      src_ext = File.extname(src).downcase
      return nil unless raster?(src_ext) && raster?(target)
      q = quality || default_quality(src_ext, target)
      cached = File.join(Config.cache, cache_name(src, target, width, q))
      return cached if File.exists?(cached)

      tmp = File.join(Config.cache, ".tmp-#{Random.rand(UInt32)}#{target}")
      args = [src]
      if w = width
        args << "-resize" << "#{w}x"
      end
      args << "-quality" << q.to_s << tmp
      status = Process.run("magick", args, output: Process::Redirect::Close,
                                           error: Process::Redirect::Close)
      unless status.success? && File.exists?(tmp)
        File.delete(tmp) if File.exists?(tmp)
        return nil
      end

      # into the store first, so an identical derivative made from two different masters is one file
      hash = Store.sha256(tmp)
      obj  = Store.object_path(hash, target)
      if File.exists?(obj)
        File.delete(tmp)
      else
        File.rename(tmp, obj)
      end
      # copy, not link -- see Store#ingest. A rule with an exception is a rule someone breaks, and
      # "nothing is ever hardlinked to an object" is worth more than the nothing this saves.
      Store.copy_cow(obj, cached) unless File.exists?(cached)
      cached
    end
  end
end
