# ── Silicon Circus · Curio — the asset server ────────────────────────────────
# `just` on its own lists every recipe.
#
# THE BINARY DOES THE WORK. This file remembers flags and sets three environment
# variables; it does not implement behaviour. Five recipes here used to shell out to
# python3 — three of them to curl a running server and ask it about files on the
# same disk — and every one of those is now a subcommand. `just find` in particular
# spliced its arguments into a shell string and then into a python literal, which
# executed whatever you typed.
#
# Recipe summaries are [doc(...)] attributes rather than plain comments: just uses
# only the LAST comment line above a recipe as its description, so a multi-line
# explanation shows up in `just --list` truncated mid-sentence.

# The single source of truth for a dev run. Every recipe that starts the binary
# exports these, so nothing depends on where the binary happens to be sitting.
port   := env_var_or_default("CURIO_PORT",   "19463")
data   := env_var_or_default("CURIO_DATA",   justfile_directory() / "data")
public := env_var_or_default("CURIO_PUBLIC", justfile_directory() / "public")
env    := "CURIO_PORT=" + port + " CURIO_DATA=" + data + " CURIO_PUBLIC=" + public
curio  := "target/release/curio"

default:
    @just --list

[doc('Run from source — the everyday one')]
dev:
    {{ env }} cargo run

[doc('Does it compile? Fastest feedback')]
check:
    cargo clippy --all-targets 2>/dev/null || cargo check

[doc('Everything, including the invariants the review found')]
test:
    cargo test

[doc('Build the deployable binary')]
build:
    cargo build --release

[doc('Run the built binary')]
run: build
    {{ env }} {{ curio }}

# Symlinked rather than copied, so a rebuild is picked up without reinstalling.
# Once this is on the PATH, `curio` finds its store by walking up from wherever you
# are, the way git finds a repository -- which is what makes an installed copy worth
# having. Without that it would default to ./data relative to the current directory
# and silently find nothing from anywhere but the project root.
[doc('Put curio on your PATH as ~/.local/bin/curio')]
install: build
    @mkdir -p ~/.local/bin
    @ln -sf "{{ justfile_directory() }}/target/release/curio" ~/.local/bin/curio
    @echo "  ~/.local/bin/curio -> target/release/curio"
    @command -v curio >/dev/null && echo "  on your PATH: $(curio --help | head -1)" \
      || echo "  NOT on your PATH -- add ~/.local/bin to it"

# ── the store ───────────────────────────────────────────────────────────────

# Prints what it intends to do with everything in watch/ -- a top-level directory
# is a collection and keeps its shape, a top-level file is one asset -- and then
# asks. `just watch yes` skips the asking.
[doc('File what is in watch/, after showing you the plan')]
watch *args: build
    {{ env }} {{ curio }} --watch {{ if args == "yes" { "--yes" } else { "" } }}

# Hashes assets/ -- but only files whose size or mtime moved -- files every replaced
# version into backup/history/ under the name it had, and refreshes the read-only
# mirror that makes the NEXT edit undoable. Asks about a rename it cannot infer.
[doc('File your edits away, and keep what they replaced')]
sync *args: build
    {{ env }} {{ curio }} --sync {{ if args == "yes" { "--yes" } else { "" } }}

[doc('Show what sync would do, and write nothing')]
plan: build
    {{ env }} {{ curio }} --sync --dry-run

# Re-hashes assets/ against the manifest. Note that this checks the files that are
# actually SERVED: the old version verified the archive copy instead, which is not
# the question anyone was asking.
[doc('Check every served asset still matches the manifest')]
verify: build
    {{ env }} {{ curio }} --verify

[doc('Share extents between byte-identical assets — `just dedup yes` to act')]
dedup *args: build
    {{ env }} {{ curio }} --dedup {{ if args == "yes" { "--yes" } else { "" } }}

[doc('State a rename so the history link survives — `just rename old new`')]
rename old new: build
    {{ env }} {{ curio }} --rename {{ old }} {{ new }}

# Renditions cost only the time to remake them. Unlike the old recipe, this also
# collects the hidden staging files: `rm -rf cache/*` is a glob, and a glob does not
# match dotfiles.
[doc('Throw away every cached rendition')]
uncache: build
    {{ env }} {{ curio }} --uncache

# du cannot see shared extents, so it counts a reflink copy in full and reports
# roughly double. df is the truth.
[doc('What the store actually costs on disk')]
du:
    @echo "apparent (du over-reports: it cannot see shared extents)"
    @du -sh "{{ data }}"/*
    @echo
    @echo "real (df)"
    @df -h "{{ data }}" | tail -1

# ── what is in there ────────────────────────────────────────────────────────

[doc('Liveness and the counts')]
ping:
    @curl -s http://127.0.0.1:{{ port }}/health && echo

# Searches what will be SERVED, derivations included: most of the park's webp are
# derived from a png master, so listing only stored files answered half the question.
[doc('Search the names — `just find cattacula night`')]
find *terms: build
    @{{ env }} {{ curio }} --find {{ terms }}

[doc('What is waiting in intake/, and what watch/ is holding')]
todo: build
    @{{ env }} {{ curio }} --todo

# ── what is actually used ───────────────────────────────────────────────────

# The procedure: forget, exercise the consumer, then read it. `requested` also lists
# anything asked for and NOT served, which is a broken reference somewhere and the one
# thing no amount of reading the consumer's source will tell you.
[doc('Clear the request table before a fresh walkthrough')]
forget: build
    @{{ env }} {{ curio }} --forget-requests

[doc('Every name ever asked for, and what was refused')]
requested: build
    @{{ env }} {{ curio }} --requested

[doc('Stored names nobody has asked for, largest first')]
unrequested: build
    @{{ env }} {{ curio }} --unrequested

[doc('Open the console')]
open:
    @xdg-open http://127.0.0.1:{{ port }} >/dev/null 2>&1 || echo "http://127.0.0.1:{{ port }}"
