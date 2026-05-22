# pxdocs-cli

Fast Rust CLI for discovering PX docs from a local `px-docs` checkout, with an optional GitHub CLI fallback.

## Install locally

```bash
cargo install --path . --force
```

Make sure Cargo's bin directory is in your `PATH`:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

Add that line to your shell startup file, such as `~/.zshrc`, `~/.bashrc`, or your shell equivalent, then reload your shell or open a new terminal.

Or run without installing:

```bash
cargo run -- <command>
```

## Setup

```bash
pxdocs setup ~/dev/px-docs
```

Without an argument, setup asks for the path interactively.

If you run a docs command before setup, the CLI asks for the local path first. Press enter without typing a path to use the GitHub fallback.

## Commands

```bash
pxdocs doctor
pxdocs decisions --guild front --limit 10
pxdocs search "react query"
pxdocs show 011 --guild front
pxdocs show docs/front-guild/decisions/011-usequery-para-consulta-de-dados.md
pxdocs search "usequery" --refresh
pxdocs search "usequery" --no-fetch
```

Local docs commands warn from the last known remote state before printing results. To keep search fast, the CLI runs `git fetch` after the command output, at most once every 10 minutes by default.

Use `--refresh` to force the post-command fetch or `--no-fetch` to skip it.

## Remote fallback

Use `--remote` to read from GitHub through `gh` instead of local files:

```bash
pxdocs decisions --guild front --remote
pxdocs show docs/front-guild/decisions/011-usequery-para-consulta-de-dados.md --remote
```

Remote listing uses GitHub tree metadata for speed. `show --remote` fetches the selected document content.
