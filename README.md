# pxdocs-cli

CLI for discovering PX docs from a local `px-docs` checkout, with an optional GitHub CLI fallback.

## Install locally

```bash
npm install -g .
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
