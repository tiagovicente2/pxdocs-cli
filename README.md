# pxdocs-cli

Fast Rust CLI for finding PX docs, decisions, guides, ADRs and RFCs from a local `px-docs` checkout, with optional GitHub fallback.

## Quick start

```bash
./install.sh
pxdocs setup ~/dev/px-docs
pxdocs search "react query"
```

If you run a docs command before setup, the CLI asks for the local `px-docs` path. Press enter without typing a path to use the GitHub fallback.

## Common commands

```bash
pxdocs search "usequery"
pxdocs decisions --guild front --limit 10
pxdocs show 011 --guild front
pxdocs show docs/front-guild/decisions/011-usequery-para-consulta-de-dados.md
pxdocs doctor
```

## Freshness and performance

Local commands print results first, then run `git fetch` after output at most once every 10 minutes.

```bash
pxdocs search "usequery" --refresh   # force post-command fetch
pxdocs search "usequery" --no-fetch  # skip fetch
```

The command still warns before output if the last known remote state says local `px-docs` is behind.

## Install

Preferred local install:

```bash
./install.sh
```

The script builds the Rust release binary and installs it to:

```txt
~/.local/bin/pxdocs
```

You can customize the bin directory:

```bash
PXDOCS_BIN_DIR="$HOME/bin" ./install.sh
```

Make sure the install directory is in your `PATH`. Add this to your shell startup file, such as `~/.zshrc`, `~/.bashrc`, or your shell equivalent:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

Alternative install with Cargo:

```bash
cargo install --path . --force
```

Or run without installing:

```bash
cargo run -- <command>
```

## Setup

```bash
pxdocs setup ~/dev/px-docs
```

Without an argument, setup asks for the path interactively.

Config is stored at:

```txt
~/.config/pxdocs/config.json
```

## Remote fallback

Use `--remote` to read from GitHub through `gh` instead of local files:

```bash
pxdocs decisions --guild front --remote
pxdocs show docs/front-guild/decisions/011-usequery-para-consulta-de-dados.md --remote
```

Remote listing uses GitHub tree metadata for speed. `show --remote` fetches the selected document content.
