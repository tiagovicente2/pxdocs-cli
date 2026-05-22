# pxdocs-cli

Fast CLI for finding PX docs from a local `px-docs` checkout, with optional GitHub fallback.

## Install

```bash
curl -fsSL https://raw.githubusercontent.com/tiagovicente2/pxdocs-cli/main/install.sh | bash
```

Be sure the install path is configured:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

Or from a local checkout:

```bash
cargo install --path . --force
```

## Setup

```bash
pxdocs setup <px-docs-path>
```

## Usage

```bash
pxdocs search "react query"
pxdocs decisions --guild front
pxdocs show 011 --guild front
pxdocs doctor
```

If setup has not been run yet, the CLI asks for the local `px-docs` path. Press enter to use GitHub fallback.

## Remote fallback

```bash
pxdocs search "usequery" --remote
pxdocs decisions --guild front --remote
```

Remote mode uses the GitHub CLI (`gh`).
