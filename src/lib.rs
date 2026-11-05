//! Starlight Catch — a small arcade game built on Bevy's app and ECS crates.
//!
//! The crate compiles to a freestanding `wasm32-unknown-unknown` module that
//! implements the Bombless `bombless-game-abi` v1 surface. It has no imports
//! and exports the required lifecycle, accessor, and `memory` symbols.
//!
//! Game logic runs on a Bevy [`App`] that is advanced exactly one fixed step
//! per [`game_update`] call. Rendering is a small software rasterizer that
//! reads ECS state and writes a tightly packed RGBA8 framebuffer.

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use std::cell::RefCell;
use std::sync::OnceLock;

const WIDTH: u32 = 320;
const HEIGHT: u32 = 180;
const FRAME_BYTES: usize = (WIDTH * HEIGHT * 4) as usize;
const CONFIG_CAP: usize = 1024;
const INPUT_CAP: usize = 1024;

const FIXED_DT: f32 = 1.0 / 60.0;

const GAME_ABI_VERSION: u32 = 1;
const GAME_OK: i32 = 0;
const GAME_ERR_INVALID_ARGUMENT: i32 = -1;
const GAME_ERR_CAPACITY: i32 = -3;

// ---------------------------------------------------------------------------
// Buffers
// ---------------------------------------------------------------------------

static mut FRAMEBUFFER: [u8; FRAME_BYTES] = [0u8; FRAME_BYTES];
static mut CONFIG_BUFFER: [u8; CONFIG_CAP] = [0u8; CONFIG_CAP];
static mut INPUT_BUFFER: [u8; INPUT_CAP] = [0u8; INPUT_CAP];

fn framebuffer() -> &'static mut [u8] {
    // SAFETY: the module is single-threaded and every ABI call is sequential.
    unsafe { &mut *core::ptr::addr_of_mut!(FRAMEBUFFER) }
}

fn config_buffer() -> &'static mut [u8] {
    // SAFETY: see `framebuffer`.
    unsafe { &mut *core::ptr::addr_of_mut!(CONFIG_BUFFER) }
}

fn input_buffer() -> &'static mut [u8] {
    // SAFETY: see `framebuffer`.
    unsafe { &mut *core::ptr::addr_of_mut!(INPUT_BUFFER) }
}

thread_local! {
    static APP: RefCell<Option<App>> = const { RefCell::new(None) };
}

// ---------------------------------------------------------------------------
// Components and resources
// ---------------------------------------------------------------------------

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Star;

#[derive(Component)]
struct Mine;

#[derive(Component)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Component)]
struct Velocity {
    x: f32,
    y: f32,
}

#[derive(Component)]
struct Radius(f32);

#[derive(Resource)]
struct Score(u32);

#[derive(Resource)]
struct Lives(u8);

#[derive(Resource)]
struct GameOver(bool);

#[derive(Resource)]
struct Invulnerable(u32);

#[derive(Resource)]
struct HeldKeys {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
}

impl Default for HeldKeys {
    fn default() -> Self {
        Self { up: false, down: false, left: false, right: false }
    }
}

#[derive(Resource)]
struct PointerTarget {
    x: f32,
    y: f32,
    active: bool,
}

impl Default for PointerTarget {
    fn default() -> Self {
        Self { x: WIDTH as f32 / 2.0, y: HEIGHT as f32 / 2.0, active: false }
    }
}

#[derive(Resource)]
struct SpawnTimer(f32);

#[derive(Resource)]
struct Random(u32);

#[derive(Resource)]
struct PendingInputs(Vec<GameInput>);

#[derive(Resource)]
struct ResetRequested(bool);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Key {
    Up,
    Down,
    Left,
    Right,
    Action,
}

#[derive(Clone, Debug)]
enum GameInput {
    KeyDown(Key),
    KeyUp(Key),
    PointerDown { x: f32, y: f32 },
    PointerUp { x: f32, y: f32 },
    PointerMove { x: f32, y: f32 },
}

// ---------------------------------------------------------------------------
// App construction
// ---------------------------------------------------------------------------

fn build_app() -> App {
    let mut app = App::new();
    app.insert_resource(Score(0))
        .insert_resource(Lives(3))
        .insert_resource(GameOver(false))
        .insert_resource(Invulnerable(0))
        .insert_resource(HeldKeys::default())
        .insert_resource(PointerTarget::default())
        .insert_resource(SpawnTimer(0.4))
        .insert_resource(Random(0x5EED_1234))
        .insert_resource(PendingInputs(Vec::new()))
        .insert_resource(ResetRequested(false))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                apply_inputs,
                player_movement,
                spawn_entities,
                move_entities,
                check_collisions,
                cleanup_entities,
                reset_game,
            ),
        );
    app
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Player,
        Position { x: WIDTH as f32 / 2.0, y: HEIGHT as f32 - 24.0 },
        Radius(9.0),
    ));
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

fn apply_inputs(
    mut queue: ResMut<PendingInputs>,
    mut held: ResMut<HeldKeys>,
    mut pointer: ResMut<PointerTarget>,
    game_over: Res<GameOver>,
    mut reset: ResMut<ResetRequested>,
) {
    for input in queue.0.drain(..) {
        match input {
            GameInput::KeyDown(key) => {
                if key == Key::Action && game_over.0 {
                    reset.0 = true;
                }
                set_key(&mut held, key, true);
            }
            GameInput::KeyUp(key) => set_key(&mut held, key, false),
            GameInput::PointerDown { x, y } => {
                pointer.x = x;
                pointer.y = y;
                pointer.active = true;
            }
            GameInput::PointerMove { x, y } => {
                if pointer.active {
                    pointer.x = x;
                    pointer.y = y;
                }
            }
            GameInput::PointerUp { .. } => pointer.active = false,
        }
    }
}

fn set_key(held: &mut HeldKeys, key: Key, value: bool) {
    match key {
        Key::Up => held.up = value,
        Key::Down => held.down = value,
        Key::Left => held.left = value,
        Key::Right => held.right = value,
        Key::Action => {}
    }
}

fn player_movement(
    mut player: Single<&mut Position, With<Player>>,
    held: Res<HeldKeys>,
    pointer: Res<PointerTarget>,
    game_over: Res<GameOver>,
) {
    if game_over.0 {
        return;
    }
    let speed = 132.0;
    let mut dx: f32 = 0.0;
    let mut dy: f32 = 0.0;
    if held.left {
        dx -= 1.0;
    }
    if held.right {
        dx += 1.0;
    }
    if held.up {
        dy -= 1.0;
    }
    if held.down {
        dy += 1.0;
    }
    if dx != 0.0 || dy != 0.0 {
        let len = (dx * dx + dy * dy).sqrt();
        player.x += dx / len * speed * FIXED_DT;
        player.y += dy / len * speed * FIXED_DT;
    } else if pointer.active {
        let dx = pointer.x - player.x;
        let dy = pointer.y - player.y;
        let dist = (dx * dx + dy * dy).sqrt();
        if dist > 0.5 {
            let step = speed * FIXED_DT;
            if step >= dist {
                player.x = pointer.x;
                player.y = pointer.y;
            } else {
                player.x += dx / dist * step;
                player.y += dy / dist * step;
            }
        }
    }
    player.x = player.x.clamp(10.0, WIDTH as f32 - 10.0);
    player.y = player.y.clamp(10.0, HEIGHT as f32 - 10.0);
}

fn spawn_entities(
    mut commands: Commands,
    mut timer: ResMut<SpawnTimer>,
    mut random: ResMut<Random>,
    game_over: Res<GameOver>,
) {
    if game_over.0 {
        return;
    }
    timer.0 -= FIXED_DT;
    if timer.0 > 0.0 {
        return;
    }
    timer.0 = 0.5 + (random.next() % 45) as f32 / 100.0;
    let x = 14.0 + (random.next() % (WIDTH - 28)) as f32;
    let fall_speed = 42.0 + (random.next() % 48) as f32;
    if random.next() % 10 < 7 {
        commands.spawn((
            Star,
            Position { x, y: -8.0 },
            Velocity { x: 0.0, y: fall_speed },
            Radius(5.0),
        ));
    } else {
        commands.spawn((
            Mine,
            Position { x, y: -8.0 },
            Velocity { x: 0.0, y: fall_speed + 18.0 },
            Radius(6.0),
        ));
    }
}

fn move_entities(
    mut query: Query<(&mut Position, &Velocity)>,
    game_over: Res<GameOver>,
    mut invulnerable: ResMut<Invulnerable>,
) {
    if game_over.0 {
        return;
    }
    for (mut position, velocity) in &mut query {
        position.x += velocity.x * FIXED_DT;
        position.y += velocity.y * FIXED_DT;
    }
    if invulnerable.0 > 0 {
        invulnerable.0 -= 1;
    }
}

fn check_collisions(
    mut commands: Commands,
    player_query: Single<(&Position, &Radius), With<Player>>,
    stars: Query<(Entity, &Position, &Radius), With<Star>>,
    mines: Query<(Entity, &Position, &Radius), With<Mine>>,
    mut score: ResMut<Score>,
    mut lives: ResMut<Lives>,
    mut game_over: ResMut<GameOver>,
    mut invulnerable: ResMut<Invulnerable>,
) {
    if game_over.0 {
        return;
    }
    let (player_position, player_radius) = player_query.into_inner();
    for (star_entity, star_position, star_radius) in &stars {
        if distance(player_position, star_position) < player_radius.0 + star_radius.0 {
            commands.entity(star_entity).despawn();
            score.0 = score.0.saturating_add(1);
        }
    }
    if invulnerable.0 == 0 {
        for (mine_entity, mine_position, mine_radius) in &mines {
            if distance(player_position, mine_position) < player_radius.0 + mine_radius.0 {
                commands.entity(mine_entity).despawn();
                lives.0 = lives.0.saturating_sub(1);
                if lives.0 == 0 {
                    game_over.0 = true;
                } else {
                    invulnerable.0 = 75;
                }
                break;
            }
        }
    }
}

fn cleanup_entities(mut commands: Commands, query: Query<(Entity, &Position)>) {
    for (entity, position) in &query {
        if position.y > HEIGHT as f32 + 20.0
            || position.x < -40.0
            || position.x > WIDTH as f32 + 40.0
        {
            commands.entity(entity).despawn();
        }
    }
}

fn reset_game(
    mut commands: Commands,
    mut score: ResMut<Score>,
    mut lives: ResMut<Lives>,
    mut game_over: ResMut<GameOver>,
    mut invulnerable: ResMut<Invulnerable>,
    mut reset: ResMut<ResetRequested>,
    mut held: ResMut<HeldKeys>,
    mut pointer: ResMut<PointerTarget>,
    stars: Query<Entity, With<Star>>,
    mines: Query<Entity, With<Mine>>,
    mut player: Single<&mut Position, With<Player>>,
) {
    if !reset.0 {
        return;
    }
    reset.0 = false;
    for entity in &stars {
        commands.entity(entity).despawn();
    }
    for entity in &mines {
        commands.entity(entity).despawn();
    }
    score.0 = 0;
    lives.0 = 3;
    game_over.0 = false;
    invulnerable.0 = 0;
    held.up = false;
    held.down = false;
    held.left = false;
    held.right = false;
    pointer.active = false;
    player.x = WIDTH as f32 / 2.0;
    player.y = HEIGHT as f32 - 24.0;
}

impl Random {
    fn next(&mut self) -> u32 {
        self.0 = self.0.wrapping_mul(1664525).wrapping_add(1013904223);
        self.0
    }
}

fn distance(a: &Position, b: &Position) -> f32 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    (dx * dx + dy * dy).sqrt()
}

// ---------------------------------------------------------------------------
// Software rasterizer
// ---------------------------------------------------------------------------

fn background() -> &'static [u8] {
    static BACKGROUND: OnceLock<Box<[u8]>> = OnceLock::new();
    BACKGROUND.get_or_init(|| {
        let mut buffer = vec![0u8; FRAME_BYTES].into_boxed_slice();
        for y in 0..HEIGHT {
            let t = y as f32 / HEIGHT as f32;
            let r = (7.0 + 9.0 * t) as u8;
            let g = (9.0 + 13.0 * t) as u8;
            let b = (20.0 + 30.0 * t) as u8;
            for x in 0..WIDTH {
                let index = ((y * WIDTH + x) * 4) as usize;
                buffer[index] = r;
                buffer[index + 1] = g;
                buffer[index + 2] = b;
                buffer[index + 3] = 255;
            }
        }
        let mut rng: u32 = 0x00C0_FFEE;
        for _ in 0..90 {
            rng = rng.wrapping_mul(1664525).wrapping_add(1013904223);
            let x = (rng % WIDTH) as usize;
            rng = rng.wrapping_mul(1664525).wrapping_add(1013904223);
            let y = (rng % HEIGHT) as usize;
            let index = (y * WIDTH as usize + x) * 4;
            let brightness = 150 + (rng as usize % 90);
            buffer[index] = brightness as u8;
            buffer[index + 1] = brightness as u8;
            buffer[index + 2] = (brightness + 20).min(255) as u8;
            buffer[index + 3] = 255;
        }
        buffer
    })
}

fn set_pixel(buffer: &mut [u8], x: i32, y: i32, color: [u8; 3]) {
    if x < 0 || y < 0 || x >= WIDTH as i32 || y >= HEIGHT as i32 {
        return;
    }
    let index = ((y as u32 * WIDTH + x as u32) * 4) as usize;
    buffer[index] = color[0];
    buffer[index + 1] = color[1];
    buffer[index + 2] = color[2];
    buffer[index + 3] = 255;
}

fn fill_rect(buffer: &mut [u8], x: i32, y: i32, width: i32, height: i32, color: [u8; 3]) {
    let x0 = x.max(0).min(WIDTH as i32);
    let y0 = y.max(0).min(HEIGHT as i32);
    let x1 = (x + width).max(0).min(WIDTH as i32);
    let y1 = (y + height).max(0).min(HEIGHT as i32);
    for yy in y0..y1 {
        for xx in x0..x1 {
            set_pixel(buffer, xx, yy, color);
        }
    }
}

fn fill_circle(buffer: &mut [u8], cx: f32, cy: f32, radius: f32, color: [u8; 3]) {
    let radius_squared = radius * radius;
    let x0 = (cx - radius).floor().max(0.0) as i32;
    let x1 = (cx + radius).ceil().min(WIDTH as f32) as i32;
    let y0 = (cy - radius).floor().max(0.0) as i32;
    let y1 = (cy + radius).ceil().min(HEIGHT as f32) as i32;
    for y in y0..y1 {
        for x in x0..x1 {
            let dx = x as f32 + 0.5 - cx;
            let dy = y as f32 + 0.5 - cy;
            if dx * dx + dy * dy <= radius_squared {
                set_pixel(buffer, x, y, color);
            }
        }
    }
}

fn draw_glow_circle(buffer: &mut [u8], cx: f32, cy: f32, radius: f32, color: [u8; 3]) {
    fill_circle(buffer, cx, cy, radius + 2.0, [color[0] / 3, color[1] / 3, color[2] / 3]);
    fill_circle(buffer, cx, cy, radius, color);
    fill_circle(buffer, cx - radius * 0.25, cy - radius * 0.25, radius * 0.4, [255, 250, 220]);
}

fn draw_mine(buffer: &mut [u8], cx: f32, cy: f32, radius: f32) {
    fill_circle(buffer, cx, cy, radius + 1.0, [90, 20, 30]);
    fill_circle(buffer, cx, cy, radius, [210, 60, 70]);
    fill_circle(buffer, cx - radius * 0.3, cy - radius * 0.3, radius * 0.35, [255, 160, 150]);
    for i in 0..8 {
        let angle = i as f32 * std::f32::consts::PI / 4.0;
        let sx = cx + angle.cos() * (radius + 1.0);
        let sy = cy + angle.sin() * (radius + 1.0);
        let ex = cx + angle.cos() * (radius + 4.0);
        let ey = cy + angle.sin() * (radius + 4.0);
        draw_line(buffer, sx, sy, ex, ey, [230, 90, 90]);
    }
}

fn draw_line(buffer: &mut [u8], x0: f32, y0: f32, x1: f32, y1: f32, color: [u8; 3]) {
    let dx = x1 - x0;
    let dy = y1 - y0;
    let steps = dx.abs().max(dy.abs()).ceil().max(1.0) as i32;
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        set_pixel(buffer, (x0 + dx * t).round() as i32, (y0 + dy * t).round() as i32, color);
    }
}

fn draw_player(buffer: &mut [u8], cx: f32, cy: f32, radius: f32) {
    let radius = radius as i32;
    let cx = cx.round() as i32;
    let cy = cy.round() as i32;
    for offset in -radius..=radius {
        let half = radius - offset.abs();
        fill_rect(buffer, cx - half, cy + offset, half * 2 + 1, 1, [70, 200, 245]);
    }
    fill_rect(buffer, cx - 2, cy - 2, 5, 5, [235, 250, 255]);
}

fn draw_heart(buffer: &mut [u8], x: i32, y: i32, color: [u8; 3]) {
    let rows = [
        "0110110", "1111111", "1111111", "0111110", "0011100", "0001000",
    ];
    for (row_index, row) in rows.iter().enumerate() {
        for (column_index, cell) in row.chars().enumerate() {
            if cell == '1' {
                set_pixel(buffer, x + column_index as i32, y + row_index as i32, color);
            }
        }
    }
}

const FONT: [[u8; 5]; 36] = [
    [0b111, 0b101, 0b101, 0b101, 0b111], // 0
    [0b010, 0b110, 0b010, 0b010, 0b111], // 1
    [0b111, 0b001, 0b111, 0b100, 0b111], // 2
    [0b111, 0b001, 0b111, 0b001, 0b111], // 3
    [0b101, 0b101, 0b111, 0b001, 0b001], // 4
    [0b111, 0b100, 0b111, 0b001, 0b111], // 5
    [0b111, 0b100, 0b111, 0b101, 0b111], // 6
    [0b111, 0b001, 0b001, 0b010, 0b010], // 7
    [0b111, 0b101, 0b111, 0b101, 0b111], // 8
    [0b111, 0b101, 0b111, 0b001, 0b111], // 9
    [0b010, 0b101, 0b111, 0b101, 0b101], // A
    [0b110, 0b101, 0b110, 0b101, 0b110], // B
    [0b011, 0b100, 0b100, 0b100, 0b011], // C
    [0b110, 0b101, 0b101, 0b101, 0b110], // D
    [0b111, 0b100, 0b110, 0b100, 0b111], // E
    [0b111, 0b100, 0b110, 0b100, 0b100], // F
    [0b011, 0b100, 0b101, 0b101, 0b011], // G
    [0b101, 0b101, 0b111, 0b101, 0b101], // H
    [0b111, 0b010, 0b010, 0b010, 0b111], // I
    [0b001, 0b001, 0b001, 0b101, 0b010], // J
    [0b101, 0b101, 0b110, 0b101, 0b101], // K
    [0b100, 0b100, 0b100, 0b100, 0b111], // L
    [0b101, 0b111, 0b111, 0b101, 0b101], // M
    [0b110, 0b101, 0b101, 0b101, 0b101], // N
    [0b010, 0b101, 0b101, 0b101, 0b010], // O
    [0b110, 0b101, 0b110, 0b100, 0b100], // P
    [0b010, 0b101, 0b101, 0b010, 0b001], // Q
    [0b110, 0b101, 0b110, 0b101, 0b101], // R
    [0b011, 0b100, 0b010, 0b001, 0b110], // S
    [0b111, 0b010, 0b010, 0b010, 0b010], // T
    [0b101, 0b101, 0b101, 0b101, 0b111], // U
    [0b101, 0b101, 0b101, 0b101, 0b010], // V
    [0b101, 0b101, 0b111, 0b111, 0b101], // W
    [0b101, 0b101, 0b010, 0b101, 0b101], // X
    [0b101, 0b101, 0b010, 0b010, 0b010], // Y
    [0b111, 0b001, 0b010, 0b100, 0b111], // Z
];

fn draw_char(buffer: &mut [u8], x: i32, y: i32, ch: char, scale: i32, color: [u8; 3]) {
    let index = match ch {
        '0'..='9' => ch as usize - '0' as usize,
        'A'..='Z' => ch as usize - 'A' as usize + 10,
        'a'..='z' => ch as usize - 'a' as usize + 10,
        _ => return,
    };
    for row in 0..5 {
        let bits = FONT[index][row];
        for column in 0..3 {
            if bits & (1 << (2 - column)) != 0 {
                fill_rect(
                    buffer,
                    x + column as i32 * scale,
                    y + row as i32 * scale,
                    scale,
                    scale,
                    color,
                );
            }
        }
    }
}

fn draw_text(buffer: &mut [u8], x: i32, y: i32, text: &str, scale: i32, color: [u8; 3]) {
    let mut cursor = x;
    for ch in text.chars() {
        draw_char(buffer, cursor, y, ch, scale, color);
        cursor += 4 * scale;
    }
}

fn draw_centered_text(buffer: &mut [u8], text: &str, center_x: i32, y: i32, scale: i32, color: [u8; 3]) {
    let width = text.chars().count() as i32 * 4 * scale - scale;
    draw_text(buffer, center_x - width / 2, y, text, scale, color);
}

fn draw_number(buffer: &mut [u8], x: i32, y: i32, mut value: u32, color: [u8; 3]) {
    if value == 0 {
        draw_char(buffer, x, y, '0', 1, color);
        return;
    }
    let mut digits = [0u32; 10];
    let mut count = 0;
    while value > 0 && count < digits.len() {
        digits[count] = value % 10;
        value /= 10;
        count += 1;
    }
    let mut cursor = x;
    for index in (0..count).rev() {
        let digit = char::from_digit(digits[index], 10).unwrap_or('0');
        draw_char(buffer, cursor, y, digit, 1, color);
        cursor += 4;
    }
}

fn render(buffer: &mut [u8], world: &mut World) {
    buffer.copy_from_slice(background());

    {
        let mut query = world.query_filtered::<(&Position, &Radius), With<Star>>();
        for (position, radius) in query.iter(world) {
            draw_glow_circle(buffer, position.x, position.y, radius.0, [255, 224, 120]);
        }
    }
    {
        let mut query = world.query_filtered::<(&Position, &Radius), With<Mine>>();
        for (position, radius) in query.iter(world) {
            draw_mine(buffer, position.x, position.y, radius.0);
        }
    }
    {
        let mut query = world.query_filtered::<(&Position, &Radius), With<Player>>();
        for (position, radius) in query.iter(world) {
            draw_player(buffer, position.x, position.y, radius.0);
        }
    }

    let score = world.resource::<Score>().0;
    draw_number(buffer, 6, 6, score, [235, 240, 255]);
    let lives = world.resource::<Lives>().0;
    for index in 0..lives {
        draw_heart(buffer, WIDTH as i32 - 12 - index as i32 * 11, 6, [245, 96, 120]);
    }

    if world.resource::<GameOver>().0 {
        fill_rect(
            buffer,
            WIDTH as i32 / 2 - 74,
            HEIGHT as i32 / 2 - 22,
            148,
            44,
            [12, 16, 28],
        );
        draw_centered_text(buffer, "GAME OVER", WIDTH as i32 / 2, HEIGHT as i32 / 2 - 16, 2, [255, 120, 130]);
        draw_centered_text(buffer, "PRESS ENTER", WIDTH as i32 / 2, HEIGHT as i32 / 2 + 8, 1, [220, 228, 245]);
    }
}

// ---------------------------------------------------------------------------
// Input parsing
// ---------------------------------------------------------------------------

fn json_string<'a>(text: &'a str, key: &str) -> Option<&'a str> {
    let pattern = format!("\"{}\"", key);
    let index = text.find(&pattern)?;
    let rest = &text[index + pattern.len()..];
    let colon = rest.find(':')?;
    let rest = &rest[colon + 1..];
    let start = rest.find('"')?;
    let rest = &rest[start + 1..];
    let end = rest.find('"')?;
    Some(&rest[..end])
}

fn json_f32(text: &str, key: &str) -> Option<f32> {
    let pattern = format!("\"{}\"", key);
    let index = text.find(&pattern)?;
    let rest = &text[index + pattern.len()..];
    let colon = rest.find(':')?;
    let rest = &rest[colon + 1..];
    let start = rest
        .find(|c: char| c == '-' || c.is_ascii_digit())?;
    let rest = &rest[start..];
    let end = rest
        .find(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-' || c == 'e' || c == 'E' || c == '+'))
        .unwrap_or(rest.len());
    rest[..end].parse().ok()
}

fn json_u32(text: &str, key: &str) -> Option<u32> {
    let pattern = format!("\"{}\"", key);
    let index = text.find(&pattern)?;
    let rest = &text[index + pattern.len()..];
    let colon = rest.find(':')?;
    let rest = rest[colon + 1..].trim_start();
    let end = rest
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(rest.len());
    rest[..end].parse().ok()
}

fn parse_input(bytes: &[u8]) -> Option<GameInput> {
    let text = std::str::from_utf8(bytes).ok()?;
    let event_type = json_string(text, "type")?;
    match event_type {
        "KEY_DOWN" => {
            let code = json_string(text, "code")?;
            let key = key_from_code(code)?;
            Some(GameInput::KeyDown(key))
        }
        "KEY_UP" => {
            let code = json_string(text, "code")?;
            let key = key_from_code(code)?;
            Some(GameInput::KeyUp(key))
        }
        "POINTER_DOWN" => {
            let x = json_f32(text, "x")?;
            let y = json_f32(text, "y")?;
            Some(GameInput::PointerDown { x, y })
        }
        "POINTER_UP" => {
            let x = json_f32(text, "x")?;
            let y = json_f32(text, "y")?;
            Some(GameInput::PointerUp { x, y })
        }
        "POINTER_MOVE" => {
            let x = json_f32(text, "x")?;
            let y = json_f32(text, "y")?;
            Some(GameInput::PointerMove { x, y })
        }
        _ => None,
    }
}

fn key_from_code(code: &str) -> Option<Key> {
    match code {
        "ArrowUp" | "KeyW" => Some(Key::Up),
        "ArrowDown" | "KeyS" => Some(Key::Down),
        "ArrowLeft" | "KeyA" => Some(Key::Left),
        "ArrowRight" | "KeyD" => Some(Key::Right),
        "Space" | "Enter" | "Escape" => Some(Key::Action),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// ABI v1 entry points
// ---------------------------------------------------------------------------

/// Validates that `ptr..ptr + len` is a legal range inside the 16 MiB sandbox.
fn range_valid(ptr: u32, len: u32) -> bool {
    ptr.checked_add(len)
        .is_some_and(|end| end <= 16 * 1024 * 1024)
}

#[no_mangle]
pub extern "C" fn game_abi_version() -> u32 {
    GAME_ABI_VERSION
}

#[no_mangle]
pub extern "C" fn game_init(config_ptr: u32, config_len: u32) -> i32 {
    if !range_valid(config_ptr, config_len) {
        return GAME_ERR_INVALID_ARGUMENT;
    }
    let bytes = unsafe { std::slice::from_raw_parts(config_ptr as *const u8, config_len as usize) };
    if let Ok(text) = std::str::from_utf8(bytes) {
        if let (Some(width), Some(height)) = (json_u32(text, "width"), json_u32(text, "height")) {
            if width != WIDTH || height != HEIGHT {
                return GAME_ERR_INVALID_ARGUMENT;
            }
        }
    }
    APP.with(|app| {
        let mut app = app.borrow_mut();
        *app = Some(build_app());
    });
    GAME_OK
}

#[no_mangle]
pub extern "C" fn game_update(_dt_micros: u32) -> i32 {
    APP.with(|app| {
        let mut app = app.borrow_mut();
        if let Some(app) = app.as_mut() {
            app.update();
        }
    });
    GAME_OK
}

#[no_mangle]
pub extern "C" fn game_input(event_ptr: u32, event_len: u32) -> i32 {
    if !range_valid(event_ptr, event_len) {
        return GAME_ERR_INVALID_ARGUMENT;
    }
    let bytes = unsafe { std::slice::from_raw_parts(event_ptr as *const u8, event_len as usize) };
    let input = match parse_input(bytes) {
        Some(input) => input,
        None => return GAME_ERR_INVALID_ARGUMENT,
    };
    APP.with(|app| {
        let mut app = app.borrow_mut();
        if let Some(app) = app.as_mut() {
            let mut queue = app.world_mut().resource_mut::<PendingInputs>();
            queue.0.push(input);
        }
    });
    GAME_OK
}

#[no_mangle]
pub extern "C" fn game_render(output_ptr: u32, output_capacity: u32) -> i32 {
    if output_capacity < FRAME_BYTES as u32 || !range_valid(output_ptr, FRAME_BYTES as u32) {
        return GAME_ERR_CAPACITY;
    }
    let buffer = framebuffer();
    APP.with(|app| {
        let mut app = app.borrow_mut();
        if let Some(app) = app.as_mut() {
            render(buffer, app.world_mut());
        }
    });
    if output_ptr != buffer.as_ptr() as u32 {
        unsafe {
            std::ptr::copy(buffer.as_ptr(), output_ptr as *mut u8, FRAME_BYTES);
        }
    }
    FRAME_BYTES as i32
}

#[no_mangle]
pub extern "C" fn game_shutdown() {
    APP.with(|app| {
        let mut app = app.borrow_mut();
        *app = None;
    });
}

#[no_mangle]
pub extern "C" fn game_get_width() -> u32 {
    WIDTH
}

#[no_mangle]
pub extern "C" fn game_get_height() -> u32 {
    HEIGHT
}

#[no_mangle]
pub extern "C" fn game_get_render_buffer_ptr() -> u32 {
    framebuffer().as_ptr() as u32
}

#[no_mangle]
pub extern "C" fn game_get_render_buffer_len() -> u32 {
    FRAME_BYTES as u32
}

#[no_mangle]
pub extern "C" fn game_get_config_buffer_ptr() -> u32 {
    config_buffer().as_ptr() as u32
}

#[no_mangle]
pub extern "C" fn game_get_config_buffer_capacity() -> u32 {
    CONFIG_CAP as u32
}

#[no_mangle]
pub extern "C" fn game_get_input_buffer_ptr() -> u32 {
    input_buffer().as_ptr() as u32
}

#[no_mangle]
pub extern "C" fn game_get_input_buffer_capacity() -> u32 {
    INPUT_CAP as u32
}
