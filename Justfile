# ── Silicon Circus · Archive — the asset server ────────────────────────────
# `just` on its own lists every recipe.
#
# Recipe summaries are [doc(...)] attributes rather than plain comments: just uses
# only the LAST comment line above a recipe as its description, so a multi-line
# explanation shows up in `just --list` truncated mid-sentence.

port := env_var_or_default("ARCHIVE_PORT", "26037")
data := env_var_or_default("ARCHIVE_DATA", justfile_directory() / "data")

default:
    @just --list

[doc('Fetch shards (Kemal)')]
deps:
    shards install

# Compiles from source on every start, so bin/ is never involved and an edit to
# src/ is live the moment you restart.
[doc('Run from source — the everyday one')]
dev:
    ARCHIVE_PORT={{ port }} crystal run src/archive.cr

[doc('Does it compile? No binary produced — fastest feedback')]
check:
    crystal build --no-codegen src/archive.cr

# --release matters: a plain `shards build` quietly writes a slower debug binary
# to the same path, which is easy to leave behind by accident.
[doc('Build the deployable binary')]
build:
    shards build --release

[doc('Run the built binary')]
run: build
    ARCHIVE_PORT={{ port }} bin/archive

# ── the store ───────────────────────────────────────────────────────────────

# Prints what it intends to do with everything in watch/ -- a top-level directory
# is a collection and keeps its shape, a top-level file is one asset -- and then
# asks before doing any of it. `just watch yes` skips the asking.
[doc('File what is in watch/, after showing you the plan')]
watch *args:
    bin/archive --watch {{ if args == "yes" { "--yes" } else { "" } }}

# Kenney-style kits ship every model referencing one shared atlas by a RELATIVE
# path (`Textures/colormap.png`), resolved against the .glb's own URL -- so a flat
# name in names/ would send it to /a/Textures/colormap.png and 404. Rebasing that
# uri to a SIBLING filename keeps it relative, so the files stay portable, while
# letting each model live loose under a tagged name and share one cached copy of
# the atlas rather than embedding 10 kB into every file. Only that one JSON string
# changes -- the BIN chunk stays byte-identical, the whole file grows by 12 bytes.
[doc('Rebase kit .glb onto flat tagged names, into watch/')]
glb-flatten src prefix *stems:
    #!/usr/bin/env python3
    import json, os, shutil, struct
    SRC, PREFIX = "{{ src }}", "{{ prefix }}"
    OUT, STEMS = "{{ data }}/watch", "{{ stems }}".split()

    def chunks(d):
        assert d[:4] == b"glTF" and struct.unpack("<I", d[4:8])[0] == 2, "not glTF 2.0 binary"
        assert struct.unpack("<I", d[8:12])[0] == len(d), "header length disagrees with file size"
        out, off = [], 12
        while off < len(d):
            ln, ty = struct.unpack("<I4s", d[off:off+8])
            out.append((ty, d[off+8:off+8+ln]))
            off += 8 + ln
        return out

    def rebuild(cs):
        body = b""
        for ty, data in cs:
            data += (b" " if ty == b"JSON" else b"\0") * ((-len(data)) % 4)
            body += struct.pack("<I4s", len(data), ty) + data
        return struct.pack("<4sII", b"glTF", 2, 12 + len(body)) + body

    os.makedirs(OUT, exist_ok=True)
    textures = set()
    for stem in STEMS:
        raw = open(os.path.join(SRC, stem + ".glb"), "rb").read()
        cs = chunks(raw)
        ty, j = cs[0]
        assert ty == b"JSON", stem + ": first chunk is not JSON"
        uris = set(i["uri"] for i in json.loads(j).get("images", []) if "uri" in i)
        assert uris, stem + ": no external image uri to rebase"
        for uri in uris:
            flat = PREFIX + "." + os.path.basename(uri)
            old = ('"' + uri + '"').encode()
            assert j.count(old) >= 1, stem + ": uri not present literally in the JSON chunk"
            j = j.replace(old, ('"' + flat + '"').encode())
            textures.add((os.path.normpath(os.path.join(SRC, uri)), flat))
        cs[0] = (b"JSON", j.rstrip(b" "))          # drop old padding; rebuild re-pads
        dest = os.path.join(OUT, PREFIX + "." + stem + ".glb")
        open(dest, "wb").write(rebuild(cs))
        back = chunks(open(dest, "rb").read())
        assert back[1:] == cs[1:], stem + ": binary chunk did not survive the rewrite"
        assert all(i["uri"].startswith(PREFIX + ".")
                   for i in json.loads(back[0][1]).get("images", []) if "uri" in i)
        print("  " + os.path.basename(dest))
    for path, flat in sorted(textures):
        shutil.copyfile(path, os.path.join(OUT, flat))
        print("  " + flat + "   (shared atlas)")
    print(str(len(STEMS)) + " models + " + str(len(textures)) + " texture staged in watch/ -- run `just watch`")

# Hashes names/ — but only files whose size or mtime moved — and files anything
# the store has not seen into objects/. This is what preserves the version you
# replaced when you touch a picture up in place.
#
# Runs the binary as it stands, like `watch` does. It used to depend on `build`,
# which is `shards build --release` and so a full LLVM pass before every routine
# sync. Tom: "we are good about staying on top of rebuilding, and if anything would
# affect those it would almost always be purposeful and we would rebuild anyway."
[doc('File any edits in names/ away into objects/')]
sync:
    bin/archive --sync

# Reads every object and checks its bytes still hash to its own filename. Should
# be impossible now that names/ are reflink copies, but "should be impossible" is
# the reason to check rather than the reason not to.
[doc('Verify every object still matches its hash')]
verify:
    #!/usr/bin/env python3
    import hashlib, os, time
    d = "{{ data }}/objects"
    bad, n, t = [], 0, time.time()
    for f in os.listdir(d):
        claimed = f.split(".")[0]
        if len(claimed) != 64: continue
        h = hashlib.sha256()
        with open(os.path.join(d, f), "rb") as fh:
            for c in iter(lambda: fh.read(1 << 20), b""): h.update(c)
        n += 1
        if h.hexdigest() != claimed: bad.append(f)
    print(f"verified {n} objects in {time.time()-t:.1f}s")
    print(f"corrupt: {len(bad)}")
    for b in bad: print("  ", b)

# Hardlinks an older layout (object/ source/ used/ vendor/ intake/) into data/.
# Nothing is moved or deleted; the old folders stay where they are.
# Runs the binary as it stands — same reasoning as `sync`.
[doc('Bring a pre-server layout in, non-destructively')]
migrate:
    bin/archive --migrate

# du cannot see shared extents, so it counts a reflink copy in full and reports
# roughly double. df is the truth.
[doc('What the store actually costs on disk')]
du:
    @echo "apparent (du over-reports: it cannot see shared extents)"
    @du -sh {{ data }}/*
    @echo
    @echo "real (df)"
    @df -h {{ data }} | tail -1

# Renditions are derived from masters and cost only the time to remake them.
[doc('Throw away every cached rendition')]
uncache:
    rm -rf {{ data }}/cache/*
    @echo "cache emptied; it rebuilds on demand"

# ── what is in there ────────────────────────────────────────────────────────

[doc('Liveness and the counts')]
ping:
    @curl -s http://127.0.0.1:{{ port }}/health && echo

[doc('Search the names — `just find cattacula night`')]
find *terms:
    @curl -s "http://127.0.0.1:{{ port }}/api/serve?limit=2000" \
      | python3 -c "import sys,json; ts='{{ terms }}'.lower().split(); \
        [print(i['url']) for i in json.load(sys.stdin)['items'] \
         if all(t in i['name'].lower() for t in ts)]"

[doc('What is waiting in intake/, and what watch/ is holding')]
todo:
    @curl -s http://127.0.0.1:{{ port }}/api/intake \
      | python3 -c "import sys,json; d=json.load(sys.stdin)['items']; \
        print(f'{len(d)} in intake'); [print('  ', i['file']) for i in d[:40]]"
    @curl -s http://127.0.0.1:{{ port }}/api/watch \
      | python3 -c "import sys,json; d=json.load(sys.stdin)['items']; \
        print(f'{len(d)} waiting in watch (run: just watch)'); \
        [print('  ', i['kind'].upper().ljust(11), i['name'], i['why'] and '-- '+i['why'] or '') for i in d]"

[doc('Open the console')]
open:
    @xdg-open http://127.0.0.1:{{ port }} >/dev/null 2>&1 || echo "http://127.0.0.1:{{ port }}"
