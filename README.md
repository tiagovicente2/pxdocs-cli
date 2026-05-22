# pxdocs-cli

CLI for discovering PX docs from a local `px-docs` checkout, with an optional GitHub CLI fallback.

## Install locally

```bash
npm install -g .
```

If `pxdocs` is not found after installing, make sure the npm global bin directory is in your `PATH`:

```bash
npm prefix -g
# add the printed path + /bin to PATH
```

Example:

```bash
export PATH="$(npm prefix -g)/bin:$PATH"
```

Or create a symlink into a directory already in `PATH`:

```bash
ln -sf "$(npm prefix -g)/bin/pxdocs" ~/.local/bin/pxdocs
```

Or run without installing:

```bash
npm start -- <command>
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
```

Every local docs command checks whether the configured repo is behind its upstream and prints a warning when it is.

## Remote fallback

Use `--remote` to read from GitHub through `gh` instead of local files:

```bash
pxdocs decisions --guild front --remote
pxdocs show docs/front-guild/decisions/011-usequery-para-consulta-de-dados.md --remote
```

Remote listing uses GitHub tree metadata for speed. `show --remote` fetches the selected document content.
