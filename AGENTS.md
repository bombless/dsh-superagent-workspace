# Instructions for coding agents

Before changing this repository, read this file and the project README.

## Project boundaries

- This repository contains one game package and its assets only.
- Do not copy or reimplement the server, Caddy, DuckDNS, webhook, or deployment workflow implementation here.
- The canonical deployment implementation is maintained in `bombless/caddy-webhook`.
- Keep `.github/workflows/deploy.yml` as the thin caller of the game-package endpoint. Do not replace it with a locally copied deployment script.

## Package contract

- The current game is a static package. `manifest.json` and `index.html` stay at the repository root, and the manifest entry is `index.html`.
- The package is published to `POST /_deploy/game`. It is not a whole-site release and must not be sent to `POST /_deploy`.
- A published game ID and version are immutable. Raise `version` in `manifest.json` before publishing changed game content again.
- The package is served from `/games/<id>/<version>/`. Use package-relative asset paths such as `./game.js`. Do not use root-absolute paths such as `/assets/game.js`.
- `pluginType` remains `vue3` for this static package. Do not add a framework unless the game actually needs one; if one is introduced, its build must be reproducible on a clean Ubuntu GitHub Actions runner and its output must still place `manifest.json` and `index.html` at the package root.
- Verify the package and inspect every generated asset URL before committing. Every referenced asset must resolve inside the package.

## Secrets and deployment safety

- Never add credentials, API keys, bearer tokens, or server environment files to source code or generated browser assets.
- The caller workflow receives `GAME_DEPLOY_WEBHOOK_URL` and `DEPLOY_TOKEN` from GitHub Actions secrets. Never hard-code either value.
- Do not add an empty POST connectivity probe. The workflow validates configuration and sends the actual package archive.
- Do not change deployment host, webhook path, secrets names, or deployment behavior without an explicit request.
- Do not trigger `workflow_dispatch` casually: it publishes the current package version to the production game library.
- Do not modify Caddy or server infrastructure from this repository. Make infrastructure changes in `bombless/caddy-webhook`.

## Completion report

When finishing a game change, report the package ID and version, whether the package-relative asset check passed, and whether a production package publication was actually triggered. Never claim publication succeeded unless the webhook or workflow confirms it.
