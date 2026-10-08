# ── Silicon Circus · curio — a web UI for DataDungeon stores ──────────────────
# `just` on its own lists every recipe. The binary does the work; this file only
# remembers the commands. Stores and their tokens are in ~/.config/curio/curio.toml.

port := env_var_or_default("CURIO_PORT", "19463")
curio := "target/release/curio"

default:
    @just --list

[doc('Run from source — the everyday one')]
dev:
    cargo run

[doc('Does it compile? Fastest feedback')]
check:
    cargo clippy --all-targets 2>/dev/null || cargo check

[doc('Every test, including curio against a fake DataDungeon')]
test:
    cargo test

[doc('Build the binary')]
build:
    cargo build --release

[doc('Run the built binary')]
run: build
    {{ curio }}

# Symlinked rather than copied, so a rebuild is picked up without reinstalling.
[doc('Put curio on your PATH as ~/.local/bin/curio')]
install: build
    @mkdir -p ~/.local/bin
    @ln -sf "{{ justfile_directory() }}/target/release/curio" ~/.local/bin/curio
    @echo "  ~/.local/bin/curio -> target/release/curio"
    @command -v curio >/dev/null && echo "  on your PATH: $(curio --version)" \
      || echo "  NOT on your PATH -- add ~/.local/bin to it"

[doc('Does every configured store answer?')]
stores: build
    @{{ curio }} --stores

[doc('Liveness')]
ping:
    @curl -s http://127.0.0.1:{{ port }}/health && echo

[doc('Open the console')]
open:
    @xdg-open http://127.0.0.1:{{ port }} >/dev/null 2>&1 || echo "http://127.0.0.1:{{ port }}"
