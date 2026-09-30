require "kemal"
require "json"
require "./config"
require "./store"
require "./derive"
require "./watch"

# Silicon Circus — the asset server.
#
# Tom: "An asset server. The archive can become a project to serve the assets, and keep the object
# and tagged hardlinks in sync." And: "Served by name. The object store is for dedup mainly."
#
# The point is not to automate the chore, it is to delete it. A repo that references
# /a/boardwalk.cattacula.night.real.webp holds no asset, converts nothing, copies nothing, and cannot
# drift from the master. There is no step between having a picture and using it.

Curio::Config.ensure_dirs

# CORS IS FOR READING, NOT FOR WRITING — AND CORS IS NOT THE CONTROL.
#
# Every park repo embeds these assets by URL from its own origin, so a wildcard on the reads is the
# whole point. A GET can only hand out bytes that are meant to be handed out; a POST changes the
# store, so the wildcard stops at GET.
#
# But dropping the header only stops the attacker READING the reply. It never stopped the request:
# a form POST is a "simple" request, needs no preflight, and /api/sync does not even look at the
# body — so it ran. And nothing validated Host, which is what makes DNS rebinding work: a page on
# evil.example with a zero TTL rebinds to 127.0.0.1, is then genuinely same-origin, and every
# content type is free. Binding to loopback stops the network. It does not stop the browser.
#
# So a mutating request must prove it came from this machine: a loopback Host, and — when the
# browser sends one — an Origin that is this server. The console is served from here, so it passes
# without knowing any of this exists.
LOOPBACK = {"127.0.0.1", "localhost", "[::1]", "::1"}

def loopback_host?(host : String) : Bool
  name = host.starts_with?('[') ? host[0, (host.index(']') || host.size - 1) + 1] : host.split(':').first
  LOOPBACK.includes?(name)
end

before_all do |env|
  if env.request.method == "GET"
    env.response.headers["Access-Control-Allow-Origin"] = "*"
  else
    unless loopback_host?(env.request.headers["Host"]?.to_s)
      halt env, 403, "curio takes writes from this machine only"
    end
    if origin = env.request.headers["Origin"]?
      unless LOOPBACK.any? { |h| origin == "http://#{h}:#{Curio::Config.port}" }
        halt env, 403, "cross-origin write refused"
      end
    end
  end
end

get "/" do |env|
  env.response.content_type = "text/html"
  env.response.headers["Cache-Control"] = "no-cache"
  File.read(File.join(Kemal.config.public_folder, "console.html"))
end

# ── serving, by name ────────────────────────────────────────────────────────
# The name is the whole address. ?w= asks for a width; the answer is made once and stored like any
# other object, so the second request is a static file read.
get "/a/*name" do |env|
  name = env.params.url["name"].to_s
  halt env, 400, "bad name" if name.includes?("..")
  asked = File.join(Curio::Config.names, name)

  # A name that is not a file may still be one we are allowed to MAKE: /a/foo.webp answered out of
  # foo.png. A real file always wins — 42 names are already published in both formats, and a master
  # somebody made by hand is not something to second-guess with an encoder.
  src = File.file?(asked) ? asked : Curio::Derive.source_for(asked)
  halt env, 404, "no such asset" unless src

  width = env.params.query["w"]?.try(&.to_i?)
  width = nil unless width && width > 0 && width <= 8192
  quality = env.params.query["q"]?.try(&.to_i?).try(&.clamp(1, 100))

  target = File.extname(asked).downcase
  path = src
  if src != asked || width
    if made = Curio::Derive.render(src, target, width, quality)
      path = made
    elsif src != asked
      # There is nothing to fall back ON: the bytes on disk are not the format that was asked for,
      # and serving a png under a .webp name would be a lie the browser believes.
      halt env, 500, "could not render #{target} from #{File.extname(src)}"
    end
    # A failed RESIZE is a different matter — the master is still a correct answer to the name, just
    # larger than asked for. Serve it whole rather than fail the page over a thumbnail.
  end
  # Content-addressed underneath, so a given URL+width is the same bytes for ever. Long cache, and a
  # weak validator for the name itself in case a name is ever repointed.
  env.response.headers["Cache-Control"] = "public, max-age=31536000"
  send_file env, path
end

# ── the console's data ──────────────────────────────────────────────────────
# WHAT WILL BE SERVED, NOT ONLY WHAT IS STORED.
#
# `items` is the index: one row per file in names/. That was the whole answer until formats started
# being derived, and then it quietly became a lie by omission — a derived rendition is never an index
# entry, because only the MASTER is stored, so /a/halloween.tent.top.webp answers 200 while
# halloween.tent.top.webp appears nowhere in this list.
#
# Every consumer then has to rebuild the derivation table to tell "curio will not serve this" from
# "curio will make this": strip the extension, match the stem, HEAD it to be sure. Boardwalk did
# exactly that, correctly, and should not have had to — the rules are ours and they belong in one
# place. So `derivable` states them as finished names, and the question "will curio serve this?"
# goes back to being set membership.
#
# `items` and `total` keep their old shape and meaning to the byte: a consumer that only knows about
# stored files reads exactly what it read before.
get "/api/serve" do |env|
  env.response.content_type = "application/json"
  q = env.params.query["q"]?.try(&.downcase) || ""
  idx = Curio::Store.load_index
  names = idx.keys.select { |n| q.empty? || n.downcase.includes?(q) }.sort
  limit = (env.params.query["limit"]?.try(&.to_i?) || 500)

  # A derived name that is ALSO published in its own right is not derivable, it is simply a name --
  # 42 stems are held as both .png and .webp, and listing those twice would be its own wrong answer.
  published = idx.keys.to_set
  derivable = [] of NamedTuple(name: String, from: String, url: String)
  idx.each_key do |n|
    ext = File.extname(n)
    Curio::Derive.targets_for(ext).each do |target|
      d = n.rchop(ext) + target
      next if published.includes?(d)
      next unless q.empty? || d.downcase.includes?(q)
      derivable << {name: d, from: n, url: "/a/#{d}"}
    end
  end
  derivable.sort_by! { |r| r[:name] }

  {
    total: names.size,
    items: names.first(limit).map { |n|
      e = idx[n]
      {name: n, hash: e.hash, size: e.size, url: "/a/#{n}"}
    },
    derivable_total: derivable.size,
    derivable:       derivable.first(limit),
  }.to_json
end

get "/api/intake" do |env|
  env.response.content_type = "application/json"
  {items: Curio::Store.intake_list.map { |e|
    {file: e[:file], size: e[:size], mtime: e[:mtime], url: "/intake/#{e[:file]}"}
  }}.to_json
end

get "/intake/*file" do |env|
  f = env.params.url["file"].to_s
  halt env, 400, "bad name" if f.includes?("..")
  p = File.join(Curio::Config.intake, f)
  halt env, 404, "no such file" unless File.file?(p)
  send_file env, p
end

# The console SHOWS the plan; it does not run it. Filing is a command Tom types, because a folder he
# is still arranging looks exactly like a folder he has finished arranging, and only he knows which.
get "/api/watch" do |env|
  env.response.content_type = "application/json"
  {items: Curio::Filing.plan.map { |i|
    {kind: i.kind.to_s.downcase, name: i.name, files: i.files, bytes: i.bytes, why: i.why}
  }}.to_json
end

# ── the one chore, both answers ─────────────────────────────────────────────
post "/api/keep" do |env|
  env.response.content_type = "application/json"
  body = env.params.json
  file = body["file"]?.to_s
  name = body["name"]?.to_s
  r = Curio::Store.keep(file, name)
  env.response.status_code = 400 unless r[:ok]
  {ok: r[:ok], hash: r[:hash], why: r[:why], url: ("/a/" + Curio::Store.safe_name(name))}.to_json
end

post "/api/trash" do |env|
  env.response.content_type = "application/json"
  body = env.params.json
  file = body["file"]?.to_s
  ok = Curio::Store.discard(Curio::Config.intake, file)
  env.response.status_code = 400 unless ok
  {ok: ok}.to_json
end

post "/api/unpublish" do |env|
  env.response.content_type = "application/json"
  ok = Curio::Store.unpublish(env.params.json["name"]?.to_s)
  env.response.status_code = 400 unless ok
  {ok: ok}.to_json
end

post "/api/sync" do |env|
  env.response.content_type = "application/json"
  Curio::Store.sync.to_json
end

get "/health" do |env|
  env.response.content_type = "application/json"
  idx = Curio::Store.load_index
  {
    ok:      true,
    version: Curio::VERSION,
    serving: idx.size,
    objects: Dir.exists?(Curio::Config.objects) ? Dir.children(Curio::Config.objects).size : 0,
    intake:  Curio::Store.intake_list.size,
    watch:   Dir.exists?(Curio::Config.watch) ? Dir.children(Curio::Config.watch).size : 0,
    held:    Curio::Filing.plan.count { |i| i.kind.held? },
  }.to_json
end

# ── bringing the old layout across ──────────────────────────────────────────
# Hardlinks only. Nothing is moved, nothing is deleted, and the old object/ source/ used/ vendor/
# stay exactly where they are until you decide to remove them — at which point the data survives in
# data/, because it was always the same bytes.
def migrate(root : String)
  Curio::Config.ensure_dirs
  linked = skipped = 0
  {"object" => Curio::Config.objects,
   "used" => Curio::Config.names, "source" => Curio::Config.names,
   "vendor" => Curio::Config.names, "intake" => Curio::Config.intake}.each do |from, to|
    src = File.join(root, from)
    next unless Dir.exists?(src)
    n = 0
    Curio::Store.walk(src) do |path, rel|
      # flatten the few nested ones onto a dotted name, which is the naming scheme anyway
      name = rel.gsub('/', '.')
      name = "#{from}.#{name}" if from == "vendor"
      dest = File.join(to, from == "object" ? File.basename(rel) : Curio::Store.safe_name(name))
      if File.exists?(dest)
        skipped += 1
      else
        # objects/ may be hardlinked -- both ends are immutable. names/ may NOT: it is the working
        # set, and a link would tie an edit there to whatever it came from.
        ok = (to == Curio::Config.objects) ? (File.link(path, dest); true) : Curio::Store.copy_cow(path, dest)
        if ok
          linked += 1
          n += 1
        else
          skipped += 1
        end
      end
    end
    puts "  #{from.ljust(8)} -> #{File.basename(to).ljust(8)} #{n} linked"
  end
  puts "#{linked} linked, #{skipped} already there or unlinkable"
  puts "nothing was moved or deleted; the old folders are untouched"
end

if ARGV.includes?("--migrate")
  migrate(File.expand_path("..", __DIR__))
  puts "\nnow run:  ./bin/curio --sync"
  exit 0
end

# ── filing what is in watch/ ────────────────────────────────────────────────
# Prints what it intends to do, then asks. --yes skips the asking, for when you already looked.
if ARGV.includes?("--watch")
  Curio::Config.ensure_dirs
  items = Curio::Filing.plan
  Curio::Filing.describe(items)
  todo = items.reject { |i| i.kind.held? }
  exit 0 if todo.empty?
  unless ARGV.includes?("--yes")
    print "\nproceed? [y/N] "
    answer = STDIN.gets.try(&.strip.downcase)
    unless answer == "y" || answer == "yes"
      puts "nothing done"
      exit 0
    end
  end
  puts
  r = Curio::Filing.apply(items)
  puts "\nfiled #{r[:filed]} (#{r[:files]} files)" +
       (r[:skipped] > 0 ? ", #{r[:skipped]} skipped" : "") + " — run `just sync` to index them"
  exit 0
end

if ARGV.includes?("--sync")
  Curio::Config.ensure_dirs
  r = Curio::Store.sync
  puts "scanned #{r[:scanned]}, hashed #{r[:hashed]}, newly stored #{r[:ingested]}, gone #{r[:dropped]}"
  exit 0
end

Kemal.config.public_folder = ENV["CURIO_PUBLIC"]? || File.expand_path("../public", __DIR__)
Kemal.config.port = Curio::Config.port
Kemal.config.host_binding = Curio::Config.bind

Kemal.run
