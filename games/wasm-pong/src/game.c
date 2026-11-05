/* WASM Pong — a Bombless ABI v1 game module.
 *
 * Fixed-point logic in centi-pixels (1/100 px). No imports, no libc, no
 * floating point in any function signature: the host validator rejects any
 * non-i32 value type anywhere in the module type section.
 */
typedef unsigned int u32;
typedef unsigned char u8;

#define W 320
#define H 180
#define SCALE 100
#define W_S (W * SCALE)
#define H_S (H * SCALE)
#define FRAME_BYTES (W * H * 4)

#define PADDLE_W_S 600
#define PADDLE_H_S 3400
#define PLAYER_X_S 1400
#define AI_X_S (W_S - 1400 - PADDLE_W_S)
#define BALL_S 600
#define WIN_SCORE 7

static u8 framebuffer[FRAME_BYTES];
static u8 config_buf[1024];
static u8 input_buf[1024];

static int ply_y;
static int ai_y;
static int ball_x;
static int ball_y;
static int ball_vx;
static int ball_vy;
static int ply_score;
static int ai_score;
static int over;
static int serve_delay;
static int key_up;
static int key_down;
static int pointer_y = -1;
static u32 rng_state;

/* LLVM loop-idiom optimization may emit calls to these. Volatile stores keep
 * the optimizer from turning a helper into a call to itself. */
void *memset(void *dst, int c, u32 n) {
  volatile u8 *d = (volatile u8 *)dst;
  for (u32 i = 0; i < n; i++) d[i] = (u8)c;
  return dst;
}
void *memcpy(void *dst, const void *src, u32 n) {
  volatile u8 *d = (volatile u8 *)dst;
  const volatile u8 *s = (const volatile u8 *)src;
  for (u32 i = 0; i < n; i++) d[i] = s[i];
  return dst;
}
void *memmove(void *dst, const void *src, u32 n) {
  volatile u8 *d = (volatile u8 *)dst;
  const volatile u8 *s = (const volatile u8 *)src;
  if (d < s) for (u32 i = 0; i < n; i++) d[i] = s[i];
  else if (d > s) for (u32 i = n; i > 0; i--) d[i - 1] = s[i - 1];
  return dst;
}

static u32 rnd(void) {
  rng_state = rng_state * 1664525u + 1013904223u;
  return rng_state;
}

/* 3x5 glyphs, five rows of three bits (bit 2 = left pixel). */
static const u8 GLYPHS[24][5] = {
  {7, 5, 5, 5, 7}, /* 0 */
  {2, 2, 2, 2, 2}, /* 1 */
  {7, 1, 7, 4, 7}, /* 2 */
  {7, 1, 7, 1, 7}, /* 3 */
  {5, 5, 7, 1, 1}, /* 4 */
  {7, 4, 7, 1, 7}, /* 5 */
  {7, 4, 7, 5, 7}, /* 6 */
  {7, 1, 1, 1, 1}, /* 7 */
  {7, 5, 7, 5, 7}, /* 8 */
  {7, 5, 7, 1, 7}, /* 9 */
  {7, 5, 7, 5, 5}, /* A */
  {7, 4, 4, 4, 7}, /* C */
  {7, 4, 7, 4, 7}, /* E */
  {7, 2, 2, 2, 7}, /* I */
  {4, 4, 4, 4, 7}, /* L */
  {5, 7, 7, 5, 5}, /* N */
  {7, 5, 5, 5, 7}, /* O */
  {7, 5, 7, 4, 4}, /* P */
  {7, 5, 7, 6, 5}, /* R */
  {7, 4, 7, 1, 7}, /* S */
  {5, 5, 5, 5, 7}, /* U */
  {5, 5, 5, 5, 2}, /* V */
  {5, 5, 7, 7, 5}, /* W */
  {5, 5, 2, 2, 2}, /* Y */
};

static int glyph_index(u8 c) {
  if (c >= '0' && c <= '9') return c - '0';
  switch (c) {
    case 'A': return 10;
    case 'C': return 11;
    case 'E': return 12;
    case 'I': return 13;
    case 'L': return 14;
    case 'N': return 15;
    case 'O': return 16;
    case 'P': return 17;
    case 'R': return 18;
    case 'S': return 19;
    case 'U': return 20;
    case 'V': return 21;
    case 'W': return 22;
    case 'Y': return 23;
  }
  return -1;
}

static void pixel(int x, int y, u8 r, u8 g, u8 b) {
  if (x < 0 || y < 0 || x >= W || y >= H) return;
  u32 i = ((u32)y * W + (u32)x) * 4;
  framebuffer[i] = r;
  framebuffer[i + 1] = g;
  framebuffer[i + 2] = b;
  framebuffer[i + 3] = 255;
}

static void rect(int x, int y, int w, int h, u8 r, u8 g, u8 b) {
  for (int yy = y; yy < y + h; yy++)
    for (int xx = x; xx < x + w; xx++) pixel(xx, yy, r, g, b);
}

static void draw_glyph(int x, int y, int idx, int scale, u8 r, u8 g, u8 b) {
  for (int row = 0; row < 5; row++) {
    u8 bits = GLYPHS[idx][row];
    for (int col = 0; col < 3; col++)
      if (bits & (4 >> col)) rect(x + col * scale, y + row * scale, scale, scale, r, g, b);
  }
}

static void draw_text(int x, int y, const char *s, int scale, u8 r, u8 g, u8 b) {
  int cx = x;
  for (const u8 *p = (const u8 *)s; *p; p++) {
    if (*p != ' ') {
      int idx = glyph_index(*p);
      if (idx >= 0) draw_glyph(cx, y, idx, scale, r, g, b);
    }
    cx += 4 * scale;
  }
}

static void draw_number_right(int x_right, int y, int n, int scale, u8 r, u8 g, u8 b) {
  char buf[3];
  int digits;
  if (n >= 10) {
    buf[0] = (char)('0' + n / 10);
    buf[1] = (char)('0' + n % 10);
    buf[2] = 0;
    digits = 2;
  } else {
    buf[0] = (char)('0' + n);
    buf[1] = 0;
    digits = 1;
  }
  int width = digits * 4 * scale - scale;
  draw_text(x_right - width, y, buf, scale, r, g, b);
}

static void serve_ball(void) {
  ball_x = (W_S - BALL_S) / 2;
  ball_y = (H_S - BALL_S) / 2;
  int dir = (rnd() & 1) ? 1 : -1;
  int vy = (int)(rnd() % 60) * 150 - 4500;
  ball_vx = 15000 * dir;
  ball_vy = vy;
}

static void reset_game(void) {
  ply_score = 0;
  ai_score = 0;
  over = 0;
  ply_y = (H_S - PADDLE_H_S) / 2;
  ai_y = (H_S - PADDLE_H_S) / 2;
  ball_x = (W_S - BALL_S) / 2;
  ball_y = (H_S - BALL_S) / 2;
  ball_vx = 0;
  ball_vy = 0;
  serve_delay = 40;
  key_up = 0;
  key_down = 0;
  pointer_y = -1;
  rng_state = 0x9E3779B9u;
}

static void update_game(u32 dt) {
  if (over) return;
  int step_units = (int)dt;

  int max_step = 42000 * step_units / 1000000;
  if (pointer_y >= 0) {
    int target = pointer_y - PADDLE_H_S / 2;
    int diff = target - ply_y;
    if (diff > max_step) diff = max_step;
    if (diff < -max_step) diff = -max_step;
    ply_y += diff;
  } else {
    int step = 24000 * step_units / 1000000;
    if (key_up) ply_y -= step;
    if (key_down) ply_y += step;
  }
  if (ply_y < 0) ply_y = 0;
  if (ply_y > H_S - PADDLE_H_S) ply_y = H_S - PADDLE_H_S;

  int ai_step = 20000 * step_units / 1000000;
  int ai_target = ball_y + BALL_S / 2 - PADDLE_H_S / 2;
  int ai_diff = ai_target - ai_y;
  int ai_limit = (ball_vx > 0) ? ai_step : ai_step / 2;
  if (ai_diff > ai_limit) ai_diff = ai_limit;
  if (ai_diff < -ai_limit) ai_diff = -ai_limit;
  ai_y += ai_diff;
  if (ai_y < 0) ai_y = 0;
  if (ai_y > H_S - PADDLE_H_S) ai_y = H_S - PADDLE_H_S;

  if (serve_delay > 0) {
    serve_delay--;
    if (serve_delay == 0) serve_ball();
    return;
  }

  ball_x += ball_vx * step_units / 1000000;
  ball_y += ball_vy * step_units / 1000000;

  if (ball_y < 0) {
    ball_y = 0;
    ball_vy = -ball_vy;
  }
  if (ball_y > H_S - BALL_S) {
    ball_y = H_S - BALL_S;
    ball_vy = -ball_vy;
  }

  if (ball_vx < 0 && ball_x <= PLAYER_X_S + PADDLE_W_S &&
      ball_x + BALL_S >= PLAYER_X_S && ball_y + BALL_S >= ply_y &&
      ball_y <= ply_y + PADDLE_H_S) {
    int hit = (ball_y + BALL_S / 2 - ply_y) / PADDLE_H_S;
    ball_vx = -ball_vx;
    if (ball_vx > 32000) ball_vx = 32000;
    ball_vy = (hit - 50) * 520 + (int)(rnd() % 80) * 5 - 200;
    ball_x = PLAYER_X_S + PADDLE_W_S;
  }
  if (ball_vx > 0 && ball_x + BALL_S >= AI_X_S &&
      ball_x <= AI_X_S + PADDLE_W_S && ball_y + BALL_S >= ai_y &&
      ball_y <= ai_y + PADDLE_H_S) {
    int hit = (ball_y + BALL_S / 2 - ai_y) / PADDLE_H_S;
    ball_vx = -ball_vx;
    if (ball_vx < -32000) ball_vx = -32000;
    ball_vy = (hit - 50) * 520 + (int)(rnd() % 80) * 5 - 200;
    ball_x = AI_X_S - BALL_S;
  }

  if (ball_x + BALL_S < 0) {
    ai_score++;
    if (ai_score >= WIN_SCORE) {
      over = 1;
    } else {
      ball_x = (W_S - BALL_S) / 2;
      ball_y = (H_S - BALL_S) / 2;
      ball_vx = 0;
      ball_vy = 0;
      serve_delay = 40;
    }
  } else if (ball_x > W_S) {
    ply_score++;
    if (ply_score >= WIN_SCORE) {
      over = 1;
    } else {
      ball_x = (W_S - BALL_S) / 2;
      ball_y = (H_S - BALL_S) / 2;
      ball_vx = 0;
      ball_vy = 0;
      serve_delay = 40;
    }
  }
}

static void render(void) {
  rect(0, 0, W, H, 12, 16, 24);
  for (int y = 0; y < H; y += 12) rect(W / 2 - 1, y, 2, 6, 60, 70, 90);

  rect(PLAYER_X_S / SCALE, ply_y / SCALE, PADDLE_W_S / SCALE, PADDLE_H_S / SCALE, 90, 220, 200);
  rect(AI_X_S / SCALE, ai_y / SCALE, PADDLE_W_S / SCALE, PADDLE_H_S / SCALE, 240, 160, 90);
  rect(ball_x / SCALE, ball_y / SCALE, BALL_S / SCALE, BALL_S / SCALE, 240, 240, 240);

  draw_number_right(152, 10, ply_score, 4, 150, 230, 210);
  draw_number_right(176, 10, ai_score, 4, 240, 190, 120);

  if (over) {
    for (int y = 0; y < H; y++) {
      for (int x = 0; x < W; x++) {
        u32 i = ((u32)y * W + (u32)x) * 4;
        framebuffer[i] = (u8)(framebuffer[i] / 3);
        framebuffer[i + 1] = (u8)(framebuffer[i + 1] / 3);
        framebuffer[i + 2] = (u8)(framebuffer[i + 2] / 3);
      }
    }
    const char *msg = (ply_score > ai_score) ? "YOU WIN" : "YOU LOSE";
    draw_text(119, 70, msg, 3, 255, 220, 120);
    draw_text(131, 100, "SPACE", 3, 180, 200, 220);
  }
}

static int matches(const u8 *p, const char *s) {
  u32 i = 0;
  while (s[i]) {
    if (p[i] != (u8)s[i]) return 0;
    i++;
  }
  return 1;
}

static int contains(const u8 *p, u32 n, const char *s) {
  u32 i = 0;
  while (s[i]) i++;
  if (i > n) return 0;
  for (u32 k = 0; k + i <= n; k++) {
    u32 j = 0;
    while (j < i && p[k + j] == (u8)s[j]) j++;
    if (j == i) return 1;
  }
  return 0;
}

/* Returns key * 100 as an integer, or -1 when the key is absent. */
static int parse_field_x100(const u8 *p, u32 n, const char *key) {
  u32 kl = 0;
  while (key[kl]) kl++;
  for (u32 i = 0; i + kl + 2 <= n; i++) {
    if (p[i] == '"' && matches(p + i + 1, key) && p[i + 1 + kl] == '"') {
      u32 j = i + kl + 2;
      if (j < n && p[j] == ':') j++;
      while (j < n && (p[j] == ' ' || p[j] == '\t')) j++;
      int sign = 1;
      int val = 0;
      int frac = 0;
      int frac_digits = 0;
      if (j < n && p[j] == '-') {
        sign = -1;
        j++;
      }
      while (j < n && p[j] >= '0' && p[j] <= '9') {
        val = val * 10 + (p[j] - '0');
        j++;
      }
      if (j < n && p[j] == '.') {
        j++;
        while (j < n && p[j] >= '0' && p[j] <= '9') {
          frac = frac * 10 + (p[j] - '0');
          frac_digits++;
          j++;
        }
      }
      int scaled = val * 100;
      if (frac_digits == 1) scaled += frac * 10;
      else if (frac_digits == 2) scaled += frac;
      else if (frac_digits > 2) scaled += frac / 10;
      return sign * scaled;
    }
  }
  return -1;
}

static void handle_input(const u8 *e, u32 len) {
  if (contains(e, len, "KEY_DOWN")) {
    if (contains(e, len, "KeyW") || contains(e, len, "ArrowUp")) key_up = 1;
    else if (contains(e, len, "KeyS") || contains(e, len, "ArrowDown")) key_down = 1;
    else if (contains(e, len, "Space") || contains(e, len, "Enter"))
      if (over) reset_game();
  } else if (contains(e, len, "KEY_UP")) {
    if (contains(e, len, "KeyW") || contains(e, len, "ArrowUp")) key_up = 0;
    else if (contains(e, len, "KeyS") || contains(e, len, "ArrowDown")) key_down = 0;
  } else if (contains(e, len, "POINTER_MOVE") || contains(e, len, "POINTER_DOWN")) {
    int y = parse_field_x100(e, len, "y");
    if (y >= 0) pointer_y = y;
  } else if (contains(e, len, "POINTER_UP") || contains(e, len, "POINTER_CANCEL")) {
    pointer_y = -1;
  }
}

__attribute__((export_name("game_abi_version"))) u32 game_abi_version(void) { return 1; }

__attribute__((export_name("game_init"))) int game_init(u32 ptr, u32 len) {
  if (ptr != (u32)(unsigned long)config_buf || len > sizeof(config_buf)) return -1;
  reset_game();
  return 0;
}

__attribute__((export_name("game_update"))) int game_update(u32 dt) {
  update_game(dt);
  return 0;
}

__attribute__((export_name("game_input"))) int game_input(u32 ptr, u32 len) {
  if (ptr != (u32)(unsigned long)input_buf || len > sizeof(input_buf)) return -1;
  handle_input(input_buf, len);
  return 0;
}

__attribute__((export_name("game_render"))) int game_render(u32 ptr, u32 cap) {
  if (ptr != (u32)(unsigned long)framebuffer || cap < FRAME_BYTES) return -3;
  render();
  return FRAME_BYTES;
}

__attribute__((export_name("game_shutdown"))) void game_shutdown(void) {}

__attribute__((export_name("game_get_width"))) u32 game_get_width(void) { return W; }

__attribute__((export_name("game_get_height"))) u32 game_get_height(void) { return H; }

__attribute__((export_name("game_get_render_buffer_ptr"))) u32 game_get_render_buffer_ptr(void) {
  return (u32)(unsigned long)framebuffer;
}

__attribute__((export_name("game_get_render_buffer_len"))) u32 game_get_render_buffer_len(void) {
  return FRAME_BYTES;
}

__attribute__((export_name("game_get_config_buffer_ptr"))) u32 game_get_config_buffer_ptr(void) {
  return (u32)(unsigned long)config_buf;
}

__attribute__((export_name("game_get_config_buffer_capacity"))) u32 game_get_config_buffer_capacity(void) {
  return sizeof(config_buf);
}

__attribute__((export_name("game_get_input_buffer_ptr"))) u32 game_get_input_buffer_ptr(void) {
  return (u32)(unsigned long)input_buf;
}

__attribute__((export_name("game_get_input_buffer_capacity"))) u32 game_get_input_buffer_capacity(void) {
  return sizeof(input_buf);
}
