# dsh-superagent-workspace

This repository is the **private source workspace** for games generated and maintained by the game-building bot.

## Starlight Catch

A small arcade game built with Rust and Bevy's app/ECS crates. Steer a collector ship, catch falling stars for points, and avoid the mines. Three lives; press Enter to restart after a game over.

The game compiles to a `wasm32-unknown-unknown` module that implements the Bombless `bombless-game-abi` v1 surface. It has no imports and renders into a 320×180 RGBA framebuffer.

**Controls**

- Arrow keys or WASD to move.
- Pointer or touch to steer the ship.
- Enter, Space, or Escape to restart after a game over.

## Publishing games

The playable package is published to the game library's `POST /_deploy/game` endpoint. The deployment webhook URL is not stored in this repository.

### Instructions for the game-building bot

Treat the deployment workflow and this section as the publishing contract:

1. Keep the complete game project in the repository root. Do not put the playable project in a separate repository or depend on GitHub Pages.
2. Keep `.github/workflows/deploy.yml`. Do not delete, rename, or replace it when generating or revising a game.
3. Keep the Rust build reproducible on a clean Ubuntu GitHub Actions runner. Commit `Cargo.toml` and `Cargo.lock`.
4. The package root must contain `manifest.json` and the declared `.wasm` entry. The manifest must satisfy the `bombless-game-abi` v1 contract.
5. The WASM module must not import host functions, WASI, or any other imports.
6. Do not put credentials, API keys, private tokens, or other secrets in source code or generated client-side files.
7. After implementing a game, verify that the production build succeeds, that the module has no imports and exports every ABI symbol, and that the manifest matches the package.

### How deployment works

- The workflow is manual: **Actions → Publish game package → Run workflow**.
- Leave the `publish` input unchecked to only build and upload the package artifact. Check it to publish the package to the production game library.
- The workflow builds the WASM module, validates the package, uploads a ZIP artifact, and (when publishing) posts it to `/_deploy/game`.
- The webhook URL is read from the `GAME_DEPLOY_WEBHOOK_URL` Actions secret. The token is read from `DEPLOY_TOKEN`.
- A published game ID and version are immutable. Raise the version before publishing changed content.

### Repository setup

In **Settings → Secrets and variables → Actions**, create these repository secrets:

- Name: `GAME_DEPLOY_WEBHOOK_URL`
  - Value: the game-package webhook URL, ending in `/_deploy/game`.
  - Must be `https`. Do not put a temporary tunnel address in the workflow or in source code; update this secret when the public URL changes.
- Name: `DEPLOY_TOKEN`
  - Value: the deployment bearer token configured on the server for that webhook.

Never print either secret in logs or commit it to the repository. The workflow sends the token only as an `Authorization: Bearer` header.

### Deployment troubleshooting

- If the workflow says `GAME_DEPLOY_WEBHOOK_URL` or `DEPLOY_TOKEN` is missing, add that Actions secret. The workflow stops before contacting the webhook.
- If the build fails, inspect the **Publish game package** workflow logs. Ensure `Cargo.toml` and `Cargo.lock` are committed and the crate builds on Linux.
- HTTP 400 means the webhook rejected the package ZIP or manifest. HTTP 401 means `DEPLOY_TOKEN` does not match the server. HTTP 409 means the game ID/version was already published. HTTP 413 means the package exceeds the upload limit.
- If the package publishes but the game is blank or unavailable, check the browser console for the WASM player error and confirm the module has no imports and exports every ABI symbol.
