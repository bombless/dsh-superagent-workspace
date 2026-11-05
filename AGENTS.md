# Instructions for coding agents

Before changing this repository, read this file and the project README.

## Project boundaries

- This repository contains one game package and its assets only.
- Do not copy or reimplement the server, Caddy, DuckDNS, webhook, or deployment workflow implementation here.
- The canonical deployment implementation is maintained in `bombless/caddy-webhook`.
- Keep `.github/workflows/deploy.yml` as the thin caller of the game-package endpoint. Do not replace it with a locally copied deployment script.

## Package contract

- The current game is a Rust Bevy WASM package. `manifest.json`, `Cargo.toml`, and `Cargo.lock` stay at the repository root.
- The package is published to `POST /_deploy/game`. It is not a whole-site release and must not be sent to `POST /_deploy`.
- The guest implements the `bombless-game-abi` v1 surface with no imports and renders a tightly packed RGBA8 framebuffer.
- A published game ID and version are immutable. Raise `version` in `manifest.json` before publishing changed game content again.
- The package is served from `/games/<id>/<version>/`. The WASM entry is `game.wasm`; do not use root-absolute paths.
- Keep the Rust build reproducible on a clean Ubuntu GitHub Actions runner. Commit `Cargo.toml` and `Cargo.lock`.
- Verify the production build, confirm the module has no imports and exports every ABI v1 symbol, and inspect the manifest before committing.

## Secrets and deployment safety

- Never add credentials, API keys, bearer tokens, or server environment files to source code or generated browser assets.
- The caller workflow receives `GAME_DEPLOY_WEBHOOK_URL` and `DEPLOY_TOKEN` from GitHub Actions secrets. Never hard-code either value.
- Do not add an empty POST connectivity probe. The workflow validates configuration and sends the actual package archive.
- Do not change deployment host, webhook path, secrets names, or deployment behavior without an explicit request.
- Do not trigger `workflow_dispatch` casually: it can publish the current package version to the production game library.
- Do not modify Caddy or server infrastructure from this repository. Make infrastructure changes in `bombless/caddy-webhook`.

## Completion report

When finishing a game change, report the package ID and version, the build command, whether the no-imports/ABI and manifest checks passed, and whether a production package publication was actually triggered. Never claim publication succeeded unless the webhook or workflow confirms it.
