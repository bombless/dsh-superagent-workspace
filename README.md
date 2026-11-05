# dsh-superagent-workspace

This repository is the **private source workspace** for games generated and maintained by the game-building bot.

## Puppy Feeding

A top-down field. It starts empty. Click or tap the grass to toss a bone, and a puppy runs in from a random edge, picks the bone up, and carries it back off the field. Toss another whenever you like; several puppies can be on the grass at once.

Open `index.html` in a browser, or play the published build at the URL below.

**Controls**

- Click or tap inside the dashed field to drop a bone.
- Fetched bones and bones still on the grass stay on screen. Nothing is saved.

## Publishing games

The playable website is published automatically under the `/app/` path of the configured static host. The deployment webhook URL is not stored in this repository.

The GitHub repository can remain **Private**. The deployed website is intentionally **public**: anyone with the URL can play it, but visitors do not get access to this repository's source code.

### Instructions for the game-building bot

Treat the deployment workflow and this section as the publishing contract:

1. Keep the complete game project in the repository root. Do not put the playable project in a separate repository or depend on GitHub Pages.
2. Keep `.github/workflows/deploy.yml`. Do not delete, rename, or replace it when generating or revising a game.
3. Make the project buildable on a clean Ubuntu GitHub Actions runner. For Node.js projects, commit `package.json` and the matching lockfile (prefer `package-lock.json`). Define a `build` script when a bundler is used.
4. The build must produce a static website. The workflow recognizes `dist/`, `build/`, or `out/` as build output directories, in that order. The selected directory must contain `index.html`. For a plain HTML/CSS/JS game with no build step, put `index.html` and its referenced assets in the repository root.
5. The site is served under the URL prefix `/app/`, not the domain root. Use relative asset URLs, or configure the framework/router for `/app/` (for example, Vite's `base: '/app/'`). Avoid hard-coded root-relative asset paths such as `/assets/game.js`, which would resolve outside `/app/`.
6. Do not put credentials, API keys, private tokens, or other secrets in source code or generated client-side files. Anything shipped to the browser is public.
7. After implementing a game, verify that the production build succeeds and that the generated `index.html` references assets correctly when hosted at `/app/`.

### How deployment works

- A push to `main` automatically builds and deploys the current game.
- You can also run **Actions → Build and deploy → Run workflow** to publish manually.
- GitHub Actions checks the deploy secrets, probes the webhook with an authenticated empty body (not the site archive), then uploads the static build as a ZIP. An empty body is not a valid release; the webhook rejects it with HTTP 400 instead of publishing.
- The webhook URL is read from the `DEPLOY_WEBHOOK_URL` Actions secret. The workflow does not fall back to a hard-coded host.
- The server atomically switches the published release after receiving a valid ZIP.
- DNS, TCP, and TLS failures are reported as curl exit codes. They are not treated as HTTP statuses returned by the webhook. TLS verification stays enabled.

### Repository setup

In **Settings → Secrets and variables → Actions**, create these repository secrets:

- Name: `DEPLOY_WEBHOOK_URL`
  - Value: the current Static Deploy webhook URL, including its path, for example `https://<host>/<path>`.
  - Must be `https`. Do not put a temporary tunnel address in the workflow or in source code; update this secret when the public URL changes.
- Name: `DEPLOY_TOKEN`
  - Value: the deployment bearer token configured on the server for that webhook.

Never print either secret in logs or commit it to the repository. The workflow sends the token only as an `Authorization: Bearer` header.

### Deployment troubleshooting

- If the workflow says `DEPLOY_WEBHOOK_URL` or `DEPLOY_TOKEN` is missing, add that Actions secret. The workflow stops before contacting the webhook.
- If the probe reports curl exit 6, 7, 28, or 35, the failure is DNS, TCP, timeout, or TLS. Fix the secret URL or the webhook's HTTPS endpoint. Do not disable TLS verification.
- If the webhook returns HTTP 401, `DEPLOY_TOKEN` does not match the server. The token value is not printed.
- HTTP 400 on the connectivity probe means the webhook rejected the empty body, which is expected. HTTP 400 on the upload means the webhook rejected the archive (`invalid_archive`). HTTP 500 means the webhook reported `internal_error`. Those are HTTP responses, distinct from the connection failures above.
- If the build fails, inspect the **Build and deploy** workflow logs. Ensure the package manifest and lockfile are committed and the build script works on Linux.
- If deployment succeeds but the page is blank or assets return 404, check that the build output contains `index.html` and that asset paths are compatible with the `/app/` URL prefix.
