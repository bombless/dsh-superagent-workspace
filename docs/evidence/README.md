# Starlight Catch — visual evidence

These captures document the game-library UI change that publishes **Starlight Catch**.

All captures use the same scenario: the live platform shell at
`https://bombless.duckdns.org/app/`, a 1280×800 viewport, the dark theme, and
the library on its initial load.

| File | State |
| --- | --- |
| `shell-before-library.png` | **Before** — the library without Starlight Catch. The registry response is intercepted to drop the new entry, reproducing the previous library state. |
| `shell-after-library.png` | **After** — the same library with the new `Starlight Catch · wasm · 1.0.0` entry. |
| `shell-after-game.png` | **After** — Starlight Catch selected and running in the WASM player, with controls granted. |
| `game-initial-frame.png` | The guest's rendered baseline frame (320×180 framebuffer, scaled 2×). |
| `game-after-input.png` | The same guest after holding `ArrowLeft` and running for a further 300 ticks: the ship has moved and more stars have spawned. |

The baseline revision is `main` at `93b95d6` (the repository before this branch).
The changed revision is the head of `agent/deploy-bevy-wasm-game-fc757552a564`.
