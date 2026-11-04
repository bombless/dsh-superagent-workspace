# Instructions for coding agents

Before changing this repository, read this file and the project README.

## Project boundaries

- This repository contains game source and game-specific assets only.
- Do not copy or reimplement the server, Caddy, DuckDNS, webhook, or deployment workflow implementation here.
- The canonical deployment implementation is maintained in `bombless/caddy-webhook`.
- Keep `.github/workflows/deploy.yml` as the thin caller of that reusable workflow. Do not replace it with a locally copied deployment script.

## Build and publish contract

- A push to `main` builds and publishes the current game. The workflow can also be run manually; either path performs a real production upload.
- The current project is plain static HTML. Its deployable output is the repository root (`output_dir: "."`) and must contain `index.html`.
- If introducing a build system, make its build reproducible on a clean Ubuntu GitHub Actions runner. Commit the package manifest and lockfile. Update the caller workflow's `build_command` and `output_dir` to match the actual build output.
- The published site is mounted at the `/app/` URL prefix. Use relative asset paths or configure the framework's public base path as `/app/`. Do not use root-absolute asset paths such as `/assets/game.js`.
- Verify the production build and inspect generated asset URLs for the `/app/` prefix before committing.

## Secrets and deployment safety

- Never add credentials, API keys, bearer tokens, or server environment files to source code or generated browser assets.
- The caller workflow receives `DEPLOY_WEBHOOK_URL` and `DEPLOY_TOKEN` from GitHub Actions secrets. Never hard-code either value.
- Do not add an empty POST connectivity probe. The reusable workflow validates configuration and diagnoses the actual archive upload. The server's `GET /health` endpoint is for health checks only.
- Do not change deployment host, webhook path, reusable workflow ref, secrets names, or deployment behavior without an explicit request.
- Do not trigger `workflow_dispatch` casually: it publishes the current game to the production website.
- Do not modify Caddy/server infrastructure from this repository. Make infrastructure changes in `bombless/caddy-webhook`.

## Completion report

When finishing a game change, report the build command and output directory, whether the `/app/` asset-path check passed, and whether a production deployment was actually triggered. Never claim deployment succeeded unless the workflow confirms it.
