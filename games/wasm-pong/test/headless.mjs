/* Headless ABI v1 verification for the WASM Pong guest module.
 * Usage: node headless.mjs <path-to-game.wasm>
 */
import { readFileSync } from 'node:fs';

const wasmPath = process.argv[2];
if (!wasmPath) {
  console.error('usage: node headless.mjs <game.wasm>');
  process.exit(2);
}

const bytes = readFileSync(wasmPath);
const module = new WebAssembly.Module(bytes);

const imports = WebAssembly.Module.imports(module);
if (imports.length !== 0) {
  throw new Error(`module must not import anything, found ${JSON.stringify(imports)}`);
}

const required = [
  'game_abi_version', 'game_init', 'game_update', 'game_input', 'game_render',
  'game_shutdown', 'game_get_width', 'game_get_height', 'game_get_render_buffer_ptr',
  'game_get_render_buffer_len', 'game_get_config_buffer_ptr', 'game_get_config_buffer_capacity',
  'game_get_input_buffer_ptr', 'game_get_input_buffer_capacity', 'memory',
];
const exported = new Map(WebAssembly.Module.exports(module).map((item) => [item.name, item.kind]));
for (const name of required) {
  if (!exported.has(name)) throw new Error(`missing export: ${name}`);
}
for (const name of required.slice(0, 14)) {
  if (exported.get(name) !== 'function') throw new Error(`export is not a function: ${name}`);
}
if (exported.get('memory') !== 'memory') throw new Error('memory export is not a memory');

function makeInstance() {
  return new WebAssembly.Instance(module, {}).exports;
}

const ex = makeInstance();
if (ex.game_abi_version() !== 1) throw new Error('unexpected ABI version');

const width = ex.game_get_width();
const height = ex.game_get_height();
if (width !== 320 || height !== 180) throw new Error(`unexpected dimensions ${width}x${height}`);

const renderLen = ex.game_get_render_buffer_len();
if (renderLen !== width * height * 4) throw new Error(`render buffer length ${renderLen}`);
const configCap = ex.game_get_config_buffer_capacity();
if (configCap < 4 || configCap > 64 * 1024) throw new Error(`config capacity ${configCap}`);
const inputCap = ex.game_get_input_buffer_capacity();
if (inputCap < 4 || inputCap > 4096) throw new Error(`input capacity ${inputCap}`);
if (ex.memory.buffer.byteLength > 16 * 65536) throw new Error('initial memory exceeds manifest limit');

const configBytes = new TextEncoder().encode(JSON.stringify({
  runtime: { display: { width, height }, wasm: { maxMemoryPages: 16, maxOutputBytes: renderLen } },
}));
if (configBytes.length > configCap) throw new Error('config does not fit guest buffer');
const configPtr = ex.game_get_config_buffer_ptr();
new Uint8Array(ex.memory.buffer, configPtr, configBytes.length).set(configBytes);
if (ex.game_init(configPtr, configBytes.length) !== 0) throw new Error('game_init failed');

function send(event) {
  const encoded = new TextEncoder().encode(JSON.stringify(event));
  if (encoded.length > inputCap) throw new Error('input event exceeds guest buffer');
  const ptr = ex.game_get_input_buffer_ptr();
  new Uint8Array(ex.memory.buffer, ptr, encoded.length).set(encoded);
  const result = ex.game_input(ptr, encoded.length);
  if (result !== 0) throw new Error(`game_input failed: ${result}`);
}

function tick(steps = 1) {
  for (let i = 0; i < steps; i++) {
    if (ex.game_update(16667) !== 0) throw new Error('game_update failed');
  }
}

function frame() {
  const ptr = ex.game_get_render_buffer_ptr();
  const written = ex.game_render(ptr, renderLen);
  if (written !== renderLen) throw new Error(`game_render returned ${written}`);
  return new Uint8Array(ex.memory.buffer.slice(ptr, ptr + renderLen));
}

function countColor(f, r, g, b) {
  let count = 0;
  for (let i = 0; i < f.length; i += 4) {
    if (f[i] === r && f[i + 1] === g && f[i + 2] === b) count++;
  }
  return count;
}

const background = countColor(frame(), 12, 16, 24);
if (background === 0) throw new Error('background was not drawn');

// Keyboard control: hold ArrowDown, then release.
send({ type: 'KEY_DOWN', code: 'ArrowDown', modifiers: 0, timestamp_micros: 0 });
tick(60);
send({ type: 'KEY_UP', code: 'ArrowDown', modifiers: 0, timestamp_micros: 1000000 });
const afterKeyDown = frame();
if (countColor(afterKeyDown, 12, 16, 24) === background) {
  throw new Error('frame did not change after keyboard input');
}

// Pointer control.
send({ type: 'POINTER_MOVE', x: 160, y: 170, code: 0, buttons: 0, modifiers: 0, timestamp_micros: 2000000 });
tick(30);
const afterPointer = frame();
if (countColor(afterPointer, 12, 16, 24) === background) {
  throw new Error('frame did not change after pointer input');
}

// Deterministic replay: a second instance with the same inputs must match.
const ex2 = makeInstance();
new Uint8Array(ex2.memory.buffer, ex2.game_get_config_buffer_ptr(), configBytes.length).set(configBytes);
if (ex2.game_init(ex2.game_get_config_buffer_ptr(), configBytes.length) !== 0) throw new Error('second game_init failed');
function send2(event) {
  const encoded = new TextEncoder().encode(JSON.stringify(event));
  const ptr = ex2.game_get_input_buffer_ptr();
  new Uint8Array(ex2.memory.buffer, ptr, encoded.length).set(encoded);
  ex2.game_input(ptr, encoded.length);
}
send2({ type: 'KEY_DOWN', code: 'ArrowDown', modifiers: 0, timestamp_micros: 0 });
for (let i = 0; i < 60; i++) ex2.game_update(16667);
send2({ type: 'KEY_UP', code: 'ArrowDown', modifiers: 0, timestamp_micros: 1000000 });
const replay = new Uint8Array(ex2.memory.buffer.slice(
  ex2.game_get_render_buffer_ptr(), ex2.game_get_render_buffer_ptr() + renderLen));
if (replay.length !== afterKeyDown.length || replay.some((v, i) => v !== afterKeyDown[i])) {
  throw new Error('replay is not deterministic');
}

// Play out a match without player input; the AI must be able to win and the
// game-over overlay must appear.
let over = false;
for (let i = 0; i < 40000; i++) {
  tick(1);
  if (i % 500 === 0) {
    const f = frame();
    // The overlay divides every channel by three.
    if (countColor(f, 12, 16, 24) === 0 && countColor(f, 4, 5, 8) > 0) { over = true; break; }
  }
}
if (!over) throw new Error('game-over overlay did not appear within the tick budget');

console.log('headless ABI verification passed');
