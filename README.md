# dsh-superagent-workspace

This repository is the **private source workspace** for games generated and maintained by the game-building bot.

## Moonlight Courier

A tiny night-time arcade game. You ride a scooter across a dreamy city, collecting glowing parcels while dodging sleepy cats, traffic cones, and puddles. You have three hearts. The road speeds up as your delivery score climbs.

Open `index.html` in a browser, or play the published build at the URL below.

**Controls**

- Move between three lanes with the left/right arrow keys or A/D.
- On a phone, tap the left or right side of the road, swipe, or use the on-screen arrows.
- Press Enter or Space, or tap the road, to start and to ride again after game over.

Score and best score stay on screen. Best score is saved in this browser.

## Publishing games

The playable website is published automatically to:

**https://bombless.duckdns.org/app/**

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
- GitHub Actions uploads the static build as a ZIP to the authenticated deployment webhook.
- The server atomically switches the published release after receiving a valid ZIP.

### Repository setup

In **Settings → Secrets and variables → Actions**, create this repository secret:

- Name: `DEPLOY_TOKEN`
- Value: the deployment token configured on the server for `https://bombless.duckdns.org/_deploy`

Never print the secret in logs or commit it to the repository.

### Deployment troubleshooting

- If the workflow says `DEPLOY_TOKEN` is missing or deployment returns HTTP 401, check that the repository Actions secret exists and matches the server-side token.
- If the build fails, inspect the **Build and deploy** workflow logs. Ensure the package manifest and lockfile are committed and the build script works on Linux.
- If deployment succeeds but the page is blank or assets return 404, check that the build output contains `index.html` and that asset paths are compatible with the `/app/` URL prefix.
