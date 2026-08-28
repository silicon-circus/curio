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

Archive::Config.ensure_dirs

before_all do |env|
  env.response.headers["Access-Control-Allow-Origin"] = "*"
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
  path = File.join(Archive::Config.names, name)
  halt env, 404, "no such asset" unless File.file?(path)

  if (w = env.params.query["w"]?)
    width = w.to_i?
    if width && width > 0 && width <= 8192
      q = (env.params.query["q"]?.try(&.to_i?) || 88).clamp(1, 100)
      if made = Archive::Derive.resized(path, width, q)
        path = made
      end
    end
  end
  # Content-addressed underneath, so a given URL+width is the same bytes for ever. Long cache, and a
  # weak validator for the name itself in case a name is ever repointed.
  env.response.headers["Cache-Control"] = "public, max-age=31536000"
  send_file env, path
end

# ── the console's data ──────────────────────────────────────────────────────
get "/api/serve" do |env|
  env.response.content_type = "application/json"
  q = env.params.query["q"]?.try(&.downcase) || ""
  idx = Archive::Store.load_index
  names = idx.keys.select { |n| q.empty? || n.downcase.includes?(q) }.sort
  limit = (env.params.query["limit"]?.try(&.to_i?) || 500)
  {
    total: names.size,
    items: names.first(limit).map { |n|
      e = idx[n]
      {name: n, hash: e.hash, size: e.size, url: "/a/#{n}"}
    },
  }.to_json
end

get "/api/intake" do |env|
  env.response.content_type = "application/json"
  {items: Archive::Store.intake_list.map { |e|
    {file: e[:file], size: e[:size], mtime: e[:mtime], url: "/intake/#{e[:file]}"}
  }}.to_json
end

get "/intake/*file" do |env|
  f = env.params.url["file"].to_s
  halt env, 400, "bad name" if f.includes?("..")
  p = File.join(Archive::Config.intake, f)
  halt env, 404, "no such file" unless File.file?(p)
  send_file env, p
end

# The console SHOWS the plan; it does not run it. Filing is a command Tom types, because a folder he
# is still arranging looks exactly like a folder he has finished arranging, and only he knows which.
get "/api/watch" do |env|
  env.response.content_type = "application/json"
  {items: Archive::Filing.plan.map { |i|
    {kind: i.kind.to_s.downcase, name: i.name, files: i.files, bytes: i.bytes, why: i.why}
  }}.to_json
end

# ── the one chore, both answers ─────────────────────────────────────────────
post "/api/keep" do |env|
  env.response.content_type = "application/json"
  body = env.params.json
  file = body["file"]?.to_s
  name = body["name"]?.to_s
  r = Archive::Store.keep(file, name)
  env.response.status_code = 400 unless r[:ok]
  {ok: r[:ok], hash: r[:hash], why: r[:why], url: ("/a/" + Archive::Store.safe_name(name))}.to_json
end

post "/api/trash" do |env|
  env.response.content_type = "application/json"
  body = env.params.json
  file = body["file"]?.to_s
  ok = Archive::Store.discard(Archive::Config.intake, file)
  env.response.status_code = 400 unless ok
  {ok: ok}.to_json
end

post "/api/unpublish" do |env|
  env.response.content_type = "application/json"
  ok = Archive::Store.unpublish(env.params.json["name"]?.to_s)
  env.response.status_code = 400 unless ok
  {ok: ok}.to_json
end

post "/api/sync" do |env|
  env.response.content_type = "application/json"
  Archive::Store.sync.to_json
end

get "/health" do |env|
  env.response.content_type = "application/json"
  idx = Archive::Store.load_index
  {
    ok:      true,
    version: Archive::VERSION,
    serving: idx.size,
    objects: Dir.exists?(Archive::Config.objects) ? Dir.children(Archive::Config.objects).size : 0,
    intake:  Archive::Store.intake_list.size,
    watch:   Dir.exists?(Archive::Config.watch) ? Dir.children(Archive::Config.watch).size : 0,
    held:    Archive::Filing.plan.count { |i| i.kind.held? },
  }.to_json
end

# ── bringing the old layout across ──────────────────────────────────────────
# Hardlinks only. Nothing is moved, nothing is deleted, and the old object/ source/ used/ vendor/
# stay exactly where they are until you decide to remove them — at which point the data survives in
# data/, because it was always the same bytes.
def migrate(root : String)
  Archive::Config.ensure_dirs
  linked = skipped = 0
  {"object" => Archive::Config.objects,
   "used" => Archive::Config.names, "source" => Archive::Config.names,
   "vendor" => Archive::Config.names, "intake" => Archive::Config.intake}.each do |from, to|
    src = File.join(root, from)
    next unless Dir.exists?(src)
    n = 0
    Archive::Store.walk(src) do |path, rel|
      # flatten the few nested ones onto a dotted name, which is the naming scheme anyway
      name = rel.gsub('/', '.')
      name = "#{from}.#{name}" if from == "vendor"
      dest = File.join(to, from == "object" ? File.basename(rel) : Archive::Store.safe_name(name))
      if File.exists?(dest)
        skipped += 1
      else
        # objects/ may be hardlinked -- both ends are immutable. names/ may NOT: it is the working
        # set, and a link would tie an edit there to whatever it came from.
        ok = (to == Archive::Config.objects) ? (File.link(path, dest); true) : Archive::Store.copy_cow(path, dest)
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
  puts "\nnow run:  ./bin/archive --sync"
  exit 0
end

# ── filing what is in watch/ ────────────────────────────────────────────────
# Prints what it intends to do, then asks. --yes skips the asking, for when you already looked.
if ARGV.includes?("--watch")
  Archive::Config.ensure_dirs
  items = Archive::Filing.plan
  Archive::Filing.describe(items)
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
  r = Archive::Filing.apply(items)
  puts "\nfiled #{r[:filed]} (#{r[:files]} files) — run `just sync` to index them"
  exit 0
end

if ARGV.includes?("--sync")
  Archive::Config.ensure_dirs
  r = Archive::Store.sync
  puts "scanned #{r[:scanned]}, hashed #{r[:hashed]}, newly stored #{r[:ingested]}, gone #{r[:dropped]}"
  exit 0
end

Kemal.config.public_folder = ENV["ARCHIVE_PUBLIC"]? || File.expand_path("../public", __DIR__)
Kemal.config.port = Archive::Config.port

Kemal.run
