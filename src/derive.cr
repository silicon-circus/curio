require "./config"
require "./store"

module Archive
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

    def raster?(ext : String) : Bool
      RASTER.includes?(ext.downcase)
    end

    # THE CACHE KEY CARRIES THE FILE'S STATE, NOT JUST ITS NAME.
    #
    # Keyed on the name alone — which is how I first wrote it — a rendition outlives the master it
    # was made from: touch up boardwalk.cattacula.night.real.png and the site keeps being handed the
    # 320-wide version of the picture you just replaced, for ever, with nothing to tell you. So the
    # key includes size and mtime. Editing anything changes at least one of them, the old key is
    # simply never asked for again, and the next request renders fresh. A stat, not a hash: this
    # happens on every request and hashing a 3 MB master to serve a 14 kB thumbnail is not a trade.
    def cache_name(src : String, width : Int32?, quality : Int32, ext : String) : String
      info = File.info(src)
      stem = File.basename(src, ext)
      parts = [stem, "s#{info.size}", "m#{info.modification_time.to_unix}"]
      parts << "w#{width}" if width
      parts << "q#{quality}" unless quality == 88
      "#{parts.join('.')}#{ext}"
    end

    # Returns the path to serve, or nil if the conversion failed. `src` is a real file on disk.
    def resized(src : String, width : Int32, quality : Int32) : String?
      ext  = File.extname(src).downcase
      return nil unless raster?(ext)
      cached = File.join(Config.cache, cache_name(src, width, quality, ext))
      return cached if File.exists?(cached)

      tmp = File.join(Config.cache, ".tmp-#{Random.rand(UInt32)}#{ext}")
      args = ["#{src}", "-resize", "#{width}x", "-quality", quality.to_s, tmp]
      status = Process.run("magick", args, output: Process::Redirect::Close,
                                           error: Process::Redirect::Close)
      unless status.success? && File.exists?(tmp)
        File.delete(tmp) if File.exists?(tmp)
        return nil
      end

      # into the store first, so an identical derivative made from two different masters is one file
      hash = Store.sha256(tmp)
      obj  = Store.object_path(hash, ext)
      if File.exists?(obj)
        File.delete(tmp)
      else
        File.rename(tmp, obj)
      end
      File.link(obj, cached) unless File.exists?(cached)
      cached
    end
  end
end
