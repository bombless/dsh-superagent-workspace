# Instructions for coding agents

Before changing this repository, read this file and the project README. For every new game task, also read the linked Bombless runtime contract below before writing code.

## Project boundaries

- This repository is the source workspace for game projects and game-specific assets.
- Do not copy or reimplement the server, Caddy, DuckDNS, webhook, iframe host, Vue runtime, bridge, message protocol, or deployment implementation here.
- The canonical game-host and deployment implementation is maintained in `bombless/caddy-webhook`.
- Keep the existing `.github/workflows/deploy.yml` as the thin caller of the reusable whole-site deployment workflow. Do not replace it with a locally copied deployment script.
- Do not change deployment infrastructure or production configuration as part of an ordinary game request.

## Choose the correct game format

The target is normally a game that runs inside the Bombless game host. For that target, implement a **Vue 3 game plugin**, not a standalone HTML game.

Before implementing a host-loaded game, read:

- `bombless/caddy-webhook/README.md`
- `bombless/caddy-webhook/docs/vue3-plugin-runtime.md`
- `bombless/caddy-webhook/schemas/game-plugin-manifest.schema.json`
- `bombless/caddy-webhook/examples/hello-vue3/` (especially its `manifest.json`, `index.html`, `src/main.ts`, and `src/Game.vue`)

If the issue explicitly requests a standalone static website/game rather than a host-loaded plugin, follow the whole-site contract in the project README instead. If the target is unclear, ask for clarification rather than silently choosing a different architecture.

## Vue 3 game-plugin contract

For a game intended to run inside the Bombless game host:

- Implement the game UI and game logic as a Vue 3 component. The package `index.html` is only the Vue app entry/bootstrap page; do not implement the game as an unrelated standalone HTML application.
- Mount the platform-provided `RuntimeShell` and the game component as shown in the official example. Do not recreate the shell or iframe bridge.
- Use the platform SDK's `useGameRuntime()` API for host integration. Do not implement your own `postMessage` protocol or access host internals.
- The exposed game component instance must implement all four lifecycle methods using Vue's `defineExpose`: `onGameStart()`, `onGamePause()`, `onGameResume()`, and `onGameEnd()`.
- Starting must initialize gameplay exactly once. Pausing must stop/pause animation loops, timers, and active input. Resuming must continue without duplicate loops. Ending/unmounting must clean up timers, animation frames, subscriptions, and event listeners. Lifecycle calls should be safe when repeated.
- Use the SDK for supported game, input, viewport, assets, audio, score, and storage features. Check the current SDK types and runtime documentation; do not assume an API that is not implemented.
- Persist user/game data through `runtime.storage.get(key)` and `runtime.storage.set(key, value)` only. Never use browser `localStorage`, IndexedDB, cookies, or another persistence mechanism to bypass the host.
- Declare every requested capability in `manifest.json`. A manifest declaration is a request, not a grant; the host may deny a capability. Request only capabilities the game actually needs.
- Do not assume arbitrary network access, backend services, account identity, or cross-game shared storage.
- Keep all package assets inside the game package and resolve URLs relative to the package/base path. Do not hard-code domain-root paths.

## Game package and manifest

A Vue 3 plugin package must include a root-level `manifest.json` that validates against the shared schema:

- `manifestVersion: 1`
- a unique valid lowercase `id` (letters, digits, hyphens; starts with a letter or digit)
- a valid SemVer `version`
- `pluginType: "vue3"`
- `entry: "index.html"`
- `runtime.apiVersion: "1"`
- a valid `runtime.display` object and a `runtime.capabilities` array

The declared entry and all referenced assets must exist in the finished package. Do not invent additional manifest fields; the schema rejects unknown properties. Increment the version when preparing a new immutable package release.

## Build and publish contract

There are two different publication paths. Do not confuse them.

### Whole-site publication

- The existing workspace `.github/workflows/deploy.yml` publishes a complete static website to `POST /_deploy`; it is not the game-plugin package publisher.
- Its current configuration uses the repository root as `output_dir` and requires a root `index.html`.
- The site is mounted at the `/app/` URL prefix. Use relative asset URLs or configure the framework's public base path as `/app/`. Never use root-absolute asset paths such as `/assets/game.js`.
- If changing the whole-site build system, make it reproducible on a clean Ubuntu GitHub Actions runner, commit the package manifest and lockfile, and update the caller workflow's `build_command` and `output_dir` to match the actual output.

### Vue 3 game-package publication

- The game-package endpoint is `POST /_deploy/game`; it accepts one package ZIP with `manifest.json` at its root. It does not replace the whole site.
- The package publisher is documented in `bombless/caddy-webhook/.github/workflows/publish-game.yml`. That workflow currently lives in the caddy-webhook repository; do not assume this workspace's `deploy.yml` invokes it.
- Build and validate the package independently. Confirm the package directory contains the root manifest, declared entry, and all assets.
- Do not trigger a production package publication unless the issue explicitly authorizes it and the required workflow/credentials are available. If no supported publication path is configured for this workspace, report that limitation; do not improvise a deployment workflow.

## Validation expectations

Before marking a game task complete:

- Run the repository's relevant tests and production build.
- For a Vue 3 plugin, validate `manifest.json` against the shared schema and verify that its declared entry and referenced assets are present.
- Verify all runtime lifecycle methods and that pause/resume/end correctly manage timers, animation frames, and subscriptions.
- Exercise storage through the SDK where the game uses persistence.
- Inspect generated asset URLs and verify they work under the intended host/package URL arrangement; for whole-site deployment, explicitly check the `/app/` prefix.
- Report commands actually run and their results. If a check could not be run, say so; never claim it passed.

## Secrets and deployment safety

- Never add credentials, API keys, bearer tokens, or server environment files to source code or generated browser assets. Anything shipped to the browser is public.
- The whole-site caller workflow receives `DEPLOY_WEBHOOK_URL` and `DEPLOY_TOKEN` from GitHub Actions secrets. Never hard-code either value.
- Do not add an empty POST connectivity probe. The reusable whole-site workflow validates configuration and diagnoses the actual archive upload. The server's `GET /health` endpoint is for health checks only.
- Do not change deployment host, webhook path, reusable workflow ref, secrets names, or deployment behavior without an explicit request.
- Do not trigger `workflow_dispatch`, push a production deployment, or publish a game package casually: these actions can publish content to the public production site.
- Do not modify Caddy/server infrastructure from this repository. Make infrastructure changes in `bombless/caddy-webhook`.

## Issue-driven workflow

- Treat the GitHub issue as the game specification. Preserve its requested gameplay, controls, visual direction, target devices, persistence, audio, and acceptance criteria.
- If a detail is genuinely blocking, ask a focused question in the issue before making an incompatible assumption. Otherwise use sensible defaults and record them in the completion report.
- Keep changes scoped to the requested game. Do not refactor unrelated games or deployment infrastructure.
- Work on an issue-specific branch and submit a pull request when the repository's agent workflow supports it. Do not merge your own work unless explicitly authorized.

## Completion report

When finishing a game change, report:

1. What gameplay/features were implemented.
2. Main game component and entry/package locations.
3. Manifest ID, version, plugin type, and requested capabilities (for a plugin).
4. Whether all four lifecycle methods are implemented and how they behave.
5. Which runtime SDK APIs are used.
6. Build command and output directory.
7. Tests, manifest validation, and asset-path checks actually run, with results.
8. Whether a production deployment/package publication was actually triggered.

Never claim deployment or publication succeeded unless the corresponding workflow confirms it.
