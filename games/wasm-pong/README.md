# WASM Pong

A small Pong game compiled to WebAssembly for the Bombless ABI v1 game library.

- Guest: `src/game.c`, built to `game.wasm` (320×180 RGBA framebuffer, no imports).
- Manifest: `manifest.json` (`wasm-pong` 1.0.0).
- Verification: `test/headless.mjs` checks the ABI, determinism, input handling, and the game-over overlay.

## Controls

- `W` / `S` or `ArrowUp` / `ArrowDown` move the left paddle.
- Pointer movement or drag also moves the left paddle.
- `Space` or `Enter` starts a new match after game over.
- First to 7 points wins.

## Build and publish

The repository's `Publish game package` workflow builds the guest with Clang/LLD,
runs the headless verification, and posts the package to `POST /_deploy/game`.
