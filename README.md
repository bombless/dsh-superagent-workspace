# dsh-superagent-workspace

This repository is the **private source workspace** for one game package published by the game-building bot.

## Pebble Match

An 8×8 board of colored pebbles. Tap a group of at least two pebbles that share a color and touch horizontally or vertically. The group disappears, pebbles above it fall down, and the score counts every pebble removed. The board ends when no group remains.

Open `index.html` in a browser, or launch the published package from the game library. The best score is kept in local browser storage.

**Controls**

- Click or tap a matching group to remove it.
- Choose **New board** to start again.

## Publishing the game

The package is published to the persistent game library, separately from the replaceable site at `/app/`. Its public files are served under `/games/<id>/<version>/`. The deployment webhook URL is not stored in this repository.

The GitHub repository can remain **Private**. The deployed package is intentionally **public**: anyone with the URL can play it, but visitors do not get access to this repository's source code.

### Instructions for the game-building bot

Treat the deployment workflow and this section as the publishing contract:

1. Keep the complete game package in the repository root. `manifest.json` and `index.html` must be at that root.
2. Keep `.github/workflows/deploy.yml`. Do not delete, rename, or replace it when revising the game.
3. Keep asset URLs package-relative, such as `./game.js`. The package is mounted at `/games/<id>/<version>/`, so a root-absolute path would leave the package.
4. Use `pluginType: "vue3"` and `entry: "index.html"` for this static package. A framework build is optional; if one is added, commit its manifest and lockfile and make the build reproducible on a clean Ubuntu GitHub Actions runner.
5. Increase `version` before publishing changed content. The server keeps every published `id`/`version` and rejects a repeated pair.
6. Do not put credentials, API keys, private tokens, or other secrets in source code or generated client-side files. Anything shipped to the browser is public.

### How deployment works

- Run **Actions → Publish game package → Run workflow** to publish the current package manually.
- GitHub Actions checks `GAME_DEPLOY_WEBHOOK_URL` and `DEPLOY_TOKEN`, creates a ZIP with `manifest.json` at its root, and posts it to `POST /_deploy/game`.
- The webhook installs that ID and version immutably and updates the public registry. It does not replace `/app/`.
- The webhook URL is read from the `GAME_DEPLOY_WEBHOOK_URL` Actions secret. The workflow does not fall back to a hard-coded host.
- DNS, TCP, and TLS failures are reported as curl exit codes. They are not treated as HTTP statuses returned by the webhook. TLS verification stays enabled.

### Repository setup

In **Settings → Secrets and variables → Actions**, create these repository secrets:

- Name: `GAME_DEPLOY_WEBHOOK_URL`
  - Value: the game-package webhook URL, including its path. It must use `https` and end in `/_deploy/game`.
- Name: `DEPLOY_TOKEN`
  - Value: the deployment bearer token configured on the server for that webhook.

Never print either secret in logs or commit it to the repository. The workflow sends the token only as an `Authorization: Bearer` header.

### Deployment troubleshooting

- If the workflow says `GAME_DEPLOY_WEBHOOK_URL` or `DEPLOY_TOKEN` is missing, add that Actions secret. The workflow stops before contacting the webhook.
- If curl exits 6, 7, 28, or 35, the failure is DNS, TCP, timeout, or TLS. Fix the secret URL or the webhook's HTTPS endpoint. Do not disable TLS verification.
- If the webhook returns HTTP 401, `DEPLOY_TOKEN` does not match the server. The token value is not printed.
- HTTP 400 means the webhook rejected the archive or manifest. HTTP 409 means this ID and version was already published. HTTP 413 means the archive exceeds the upload limit.
- If the package publishes but assets return 404, check that every asset URL is relative to the package.
