use macroquad::prelude::*;

pub const SCREEN_W: f32 = 1280.0;
pub const SCREEN_H: f32 = 720.0;

#[cfg(target_arch = "wasm32")]
extern "C" {
    fn js_load_hi() -> i32;
    fn js_save_hi(s: i32);

    fn js_set_hud(score: i32, combo: f32, hi: i32, dist: i32);
    // 1 = playing, 2 = paused, 3 = game over
    // For game over (3), score and hi are used for display
    fn js_set_screen(screen: i32, score: i32, hi: i32);
}

#[cfg(not(target_arch = "wasm32"))]
unsafe fn js_load_hi() -> i32 {
    0
}

#[cfg(not(target_arch = "wasm32"))]
unsafe fn js_save_hi(_s: i32) {}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn js_set_hud(_score: i32, _combo: f32, _hi: i32, _dist: i32) {}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn js_set_screen(_screen: i32, _score: i32, _hi: i32) {}

// Base floor line (the terrain "top" height will be above this).
const GROUND_Y: f32 = SCREEN_H - 120.0;

const PLAYER_W: f32 = 42.0;
const PLAYER_H: f32 = 42.0;

// Player sprite drawn from Kenney's `blockPack_spritesheet.png` atlas.
// Source coordinates come from `Spritesheet/blockPack_spritesheet.xml`.
const PLAYER_SPRITE_SRC_X: f32 = 564.0;
const PLAYER_SPRITE_SRC_Y: f32 = 164.0;
const PLAYER_SPRITE_SRC_W: f32 = 16.0;
const PLAYER_SPRITE_SRC_H: f32 = 25.0;

// We treat these as "per frame at ~60fps" values and scale by `dt_factor = dt * 60.0`.
const GRAVITY_PER_FRAME: f32 = 0.5;
const JUMP_FORCE_PER_FRAME: f32 = -13.0;

const RUN_SPEED_PER_FRAME: f32 = 4.0;
const SPEED_INCREASE_PER_FRAME: f32 = 0.001;
const MAX_SPEED_PER_FRAME: f32 = 16.0;

// Phase 3: scoring + juice.
const SCORE_CLEAN_LANDING_IMPACT_MAX: f32 = 10.0;
const COMBO_INCREMENT: f32 = 0.5;
const SCORE_DISTANCE_MULT: f32 = 1.0;

// Backflip rotation (radians per frame @ ~60fps).
// 8.0 deg/frame in the project context => ~0.1396 rad/frame.
const BACKFLIP_ROT_PER_FRAME: f32 = 8.0_f32.to_radians();

// Kenney atlas tiles for visuals (from Spritesheet/blockPack_spritesheet.xml).
const TILE_W: f32 = 64.0;
const TILE_H: f32 = 74.0;

// tileGrass.png
const GRASS_SRC_X: f32 = 256.0;
const GRASS_SRC_Y: f32 = 220.0;

// tileStone.png (used for rock obstacles)
const STONE_SRC_X: f32 = 64.0;
const STONE_SRC_Y: f32 = 366.0;

// ============================================================================
// SNOW TILE DEFINITIONS (from Kenney blockPack_spritesheet.xml)
// Position tiles left-to-right for sensible sequencing
// ============================================================================

// Flat snow tile
const SNOW_FLAT_X: f32 = 128.0;
const SNOW_FLAT_Y: f32 = 220.0;

// Slope down-right (/) - terrain descending to the right
const SNOW_SLOPE_DOWN_X: f32 = 128.0;
const SNOW_SLOPE_DOWN_Y: f32 = 146.0;

// Slope up-left (\) - terrain ascending to the left
const SNOW_SLOPE_LEFT_X: f32 = 128.0;
const SNOW_SLOPE_LEFT_Y: f32 = 74.0;

// Rounded slope left (smooth transition from flat to up-left)
const SNOW_SLOPE_LEFT_ROUND_X: f32 = 128.0;
const SNOW_SLOPE_LEFT_ROUND_Y: f32 = 0.0;

// Slope up-right (/) - terrain ascending to the right
const SNOW_SLOPE_RIGHT_X: f32 = 64.0;
const SNOW_SLOPE_RIGHT_Y: f32 = 514.0;

// Rounded slope right (smooth transition from flat to up-right)
const SNOW_SLOPE_RIGHT_ROUND_X: f32 = 64.0;
const SNOW_SLOPE_RIGHT_ROUND_Y: f32 = 440.0;

// Particles.
const DUST_SPAWN_ON_LAND_COUNT: usize = 14;
const SPEED_PARTICLE_VX_THRESHOLD: f32 = 11.0;

// Terrain sampling + slope tuning.
const SLOPE_GAIN: f32 = 0.02; // applied to |delta_height| scaled by current velocity

// New rule: don't make the starting area hostile.
// Rock spawning is gated until the player has earned at least this many points.
const ROCK_SPAWN_MIN_SCORE: f32 = 200.0;

// How far ahead/behind we keep terrain + rocks generated/drawn.
const TERRAIN_AHEAD_SCREENS: f32 = 3.0;
const TERRAIN_BEHIND_SCREENS: f32 = 1.25;

// Initial camera can start at negative world-x, while terrain generation
// originally started at world-x >= 0. Generate "to the left" so the first
// frames already have terrain (avoids ~1s pop-in).
const INITIAL_TERRAIN_BACK_X: i32 = 1600;

// Death threshold (falling).
const DEATH_Y: f32 = SCREEN_H + 180.0;

// ============================================================================
// SPECTACLE CONSTANTS (viral appeal, game feel, juice)
// ============================================================================

// Entrance animation
const ENTRANCE_START_Y: f32 = -80.0;         // player starts above screen
const ENTRANCE_SLAM_DURATION: f32 = 0.35;    // seconds for drop-in bounce
const ENTRANCE_SHAKE_DURATION: f32 = 0.08;   // seconds
const ENTRANCE_SHAKE_INTENSITY: f32 = 0.012; // camera shake on land

// Screen shake
const SHAKE_DECAY: f32 = 8.0;                // how fast shake fades (per second)
const LANDING_SHAKE_BASE: f32 = 0.003;       // base shake on landing
const LANDING_SHAKE_VEL_MULT: f32 = 0.001;   // extra shake per velocity unit
const CRASH_SHAKE_INTENSITY: f32 = 0.025;    // shake on death
const BACKFLIP_FINISH_SHAKE: f32 = 0.004;    // subtle shake on clean backflip

// Flash effects
const FLASH_DURATION: f32 = 0.12;            // seconds for hit/combo flash
const COMBO_FLASH_ALPHA: f32 = 0.18;         // max alpha for combo increase
const DEATH_FLASH_ALPHA: f32 = 0.35;         // flash on game over

// Fade transitions
const FADE_SPEED: f32 = 2.5;                 // fade alpha change per second

// Visual feedback thresholds
const CLEAN_BACKFILL_ROT_THRESHOLD: f32 = std::f32::consts::PI * 1.5; // min rotation for "backflip"

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GameState {
    Playing,
    Paused,
    GameOver,
}

#[derive(Debug, Clone, Copy)]
struct Player {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    vx: f32,
    vy: f32,
    on_ground: bool,

    // Phase 3: backflip + landing/combo tracking.
    angle: f32,
    backflip_happened: bool, // true once while airborne hold-Jump started
}

#[derive(Debug, Clone, Copy)]
struct TerrainCol {
    y: f32,      // terrain top y in screen/world coordinates
    solid: bool, // false => a chasm column
    tile: SnowTile,
}

// Snow tile types for sensible terrain generation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SnowTile {
    Flat,              // Flat snow tile
    SlopeDown,         // Slope descending to right (/)
    SlopeLeft,         // Slope ascending to left (\)
    SlopeLeftRound,    // Rounded transition to slope left
    SlopeRight,        // Slope ascending to right (/)
    SlopeRightRound,   // Rounded transition to slope right
}

#[derive(Debug, Clone, Copy)]
struct Rock {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

#[derive(Debug, Clone, Copy)]
struct Particle {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    life: f32,
    max_life: f32,
    size: f32,
    color: Color,
}

pub struct Game {
    state: GameState,
    player_tex: Texture2D,
    bg_tex: Texture2D,
    player: Player,
    cam_x: f32,
    time: f32,

    // Phase 3: scoring.
    combo: f32,
    score: f32,
    distance_travelled: f32,
    hi_score: i32,

    // Procedural terrain, stored as 1 column per X pixel.
    terrain_start_x: i32,
    terrain: Vec<TerrainCol>,
    next_gen_x: i32,

    gap_until_x: i32,
    next_gap_at_x: i32,

    // "Chaining sine waves": we keep a segment with 3 sine components and
    // resample amplitude/frequency at segment boundaries.
    wave_seg_end_x: i32,
    wave_amp_a: f32,
    wave_freq_a: f32,
    wave_phase_a: f32,
    wave_amp_b: f32,
    wave_freq_b: f32,
    wave_phase_b: f32,
    wave_amp_c: f32,
    wave_freq_c: f32,
    wave_phase_c: f32,

    last_rock_x: i32,
    rocks: Vec<Rock>,

    particles: Vec<Particle>,
    last_speed_particle_t: f32,

    // Spectacle: screen shake
    shake_intensity: f32,  // current shake magnitude (0 = none)
    shake_offset_x: f32,   // current frame offset
    shake_offset_y: f32,

    // Spectacle: flash effect
    flash_alpha: f32,      // current flash opacity (0-1)
    flash_color: Color,    // flash color (usually white or accent)

    // Spectacle: fade transitions
    fade_alpha: f32,       // 0 = no fade, 1 = fully black
    fade_state: FadeState,

    // Spectacle: entrance animation
    entrance_timer: f32,   // time since entrance started
    is_in_entrance: bool,  // true while entrance animation plays

    // Tile-based terrain generation
    last_tile: SnowTile,
    last_terrain_y: f32,   // track previous height for slope detection
    slope_run_length: i32, // how many columns of current slope
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FadeState {
    None,           // No fade active
    FadingOut,      // Going to black (screen transition out)
    FadingIn,       // Coming from black (screen transition in)
    HoldBlack,      // Holding at full black for a moment
}

impl Game {
    pub fn new(player_tex: Texture2D, bg_tex: Texture2D) -> Self {
        let mut g = Self {
            state: GameState::Playing,
            player_tex,
            bg_tex,
            player: Player {
                x: 0.0,
                y: 0.0,
                w: PLAYER_W,
                h: PLAYER_H,
                vx: RUN_SPEED_PER_FRAME,
                vy: 0.0,
                on_ground: true,
                angle: 0.0,
                backflip_happened: false,
            },
            cam_x: 0.0,
            time: 0.0,

            combo: 1.0,
            score: 0.0,
            distance_travelled: 0.0,
            hi_score: unsafe { js_load_hi() },

            terrain_start_x: 0,
            terrain: Vec::new(),
            next_gen_x: 0,

            gap_until_x: -1,
            next_gap_at_x: 900,

            wave_seg_end_x: 0,
            wave_amp_a: 0.0,
            wave_freq_a: 0.0,
            wave_phase_a: 0.0,
            wave_amp_b: 0.0,
            wave_freq_b: 0.0,
            wave_phase_b: 0.0,
            wave_amp_c: 0.0,
            wave_freq_c: 0.0,
            wave_phase_c: 0.0,

            last_rock_x: -99999,
            rocks: Vec::new(),

            particles: Vec::new(),
            last_speed_particle_t: 0.0,

            shake_intensity: 0.0,
            shake_offset_x: 0.0,
            shake_offset_y: 0.0,

            flash_alpha: 0.0,
            flash_color: WHITE,

            fade_alpha: 0.0,
            fade_state: FadeState::None,

            entrance_timer: 0.0,
            is_in_entrance: false,

            last_tile: SnowTile::Flat,
            last_terrain_y: GROUND_Y,
            slope_run_length: 0,
        };

        g.reset();
        g
    }

    fn reset(&mut self) {
        self.state = GameState::Playing;
        self.time = 0.0;

        self.combo = 1.0;
        self.score = 0.0;
        self.distance_travelled = 0.0;

        self.player.angle = 0.0;
        self.player.backflip_happened = false;

        self.terrain_start_x = -INITIAL_TERRAIN_BACK_X;
        self.terrain.clear();
        self.next_gen_x = self.terrain_start_x;

        self.gap_until_x = -1;
        self.next_gap_at_x = 900 + rand::gen_range(0, 400);

        self.wave_seg_end_x = 0;
        self.choose_new_wave_segment(self.next_gen_x);

        self.last_rock_x = -99999;
        self.rocks.clear();

        // Reset tile tracking
        self.last_tile = SnowTile::Flat;
        self.last_terrain_y = GROUND_Y;
        self.slope_run_length = 0;

        // Pick a start X and then ensure it's over a solid terrain column.
        let desired_center_x = (260.0 + PLAYER_W * 0.5).floor() as i32;
        self.ensure_generated_until(desired_center_x + (SCREEN_W * 2.0) as i32);

        let mut center_x = desired_center_x;
        let mut tries = 0;
        while tries < 200 {
            let ok = self
                .terrain_at(center_x)
                .map(|c| c.solid)
                .unwrap_or(false);
            if ok {
                break;
            }
            center_x += 1;
            self.ensure_generated_until(center_x + 50);
            tries += 1;
        }

        let ground_y = self
            .terrain_at(center_x)
            .map(|c| c.y)
            .unwrap_or(GROUND_Y);

        // Spectacle: entrance animation - start player above screen
        self.player.x = center_x as f32 - PLAYER_W * 0.5;
        self.player.y = ENTRANCE_START_Y;
        self.player.vx = RUN_SPEED_PER_FRAME;
        self.player.vy = 0.0;
        self.player.on_ground = false;  // will land during entrance
        self.player.angle = 0.0;
        self.player.backflip_happened = false;

        // Store target Y for entrance animation
        self.cam_x = self.player.x - SCREEN_W * 0.35;

        self.particles.clear();
        self.last_speed_particle_t = 0.0;

        // Spectacle: start entrance animation and fade in
        self.entrance_timer = 0.0;
        self.is_in_entrance = true;
        self.trigger_fade_in();
    }

    fn terrain_at(&self, world_x: i32) -> Option<TerrainCol> {
        let idx = world_x - self.terrain_start_x;
        if idx < 0 {
            return None;
        }
        let idx = idx as usize;
        self.terrain.get(idx).copied()
    }

    fn ensure_generated_until(&mut self, world_x_max: i32) {
        while self.next_gen_x <= world_x_max {
            self.generate_one_column();
            self.next_gen_x += 1;
        }
    }

    fn choose_new_wave_segment(&mut self, x: i32) {
        // Segment length increases slightly with distance (so we get fewer "spiky"
        // discontinuities, but still get more difficulty as you go).
        let difficulty = (x.max(0) as f32) / 4500.0;
        let steep = 1.0 + difficulty * 0.02;

        let seg_len = rand::gen_range(240, 520).max(1);
        self.wave_seg_end_x = x + seg_len;

        let amp_base = 18.0 + difficulty * 18.0;

        self.wave_amp_a = amp_base * 0.95 * steep * rand::gen_range(0.7, 1.3);
        self.wave_amp_b = amp_base * 0.60 * steep * rand::gen_range(0.7, 1.25);
        self.wave_amp_c = amp_base * 0.35 * steep * rand::gen_range(0.7, 1.3);

        // Convert periods -> frequencies.
        // Higher `steep` => shorter periods => steeper hills.
        let period_a = rand::gen_range(240.0, 520.0) / steep;
        let period_b = rand::gen_range(170.0, 380.0) / steep;
        let period_c = rand::gen_range(110.0, 260.0) / steep;

        self.wave_freq_a = std::f32::consts::TAU / period_a;
        self.wave_freq_b = std::f32::consts::TAU / period_b;
        self.wave_freq_c = std::f32::consts::TAU / period_c;

        self.wave_phase_a = rand::gen_range(0.0, std::f32::consts::TAU);
        self.wave_phase_b = rand::gen_range(0.0, std::f32::consts::TAU);
        self.wave_phase_c = rand::gen_range(0.0, std::f32::consts::TAU);
    }

    fn generate_one_column(&mut self) {
        let x = self.next_gen_x;
        let difficulty = (x.max(0) as f32) / 4500.0;

        if x >= self.wave_seg_end_x {
            self.choose_new_wave_segment(x);
        }

        // Chasm scheduling
        let mut solid = true;
        if x >= self.next_gap_at_x {
            // Start a gap (chasms widen as difficulty increases).
            let gap_min = 70.0 + difficulty * 6.0;
            let gap_max = gap_min + 90.0 + difficulty * 18.0;
            let gap_max = gap_max.min(320.0);

            let gap_len = rand::gen_range(gap_min, gap_max).max(18.0) as i32;
            self.gap_until_x = x + gap_len;
            self.next_gap_at_x =
                self.gap_until_x + rand::gen_range(240.0_f32, 520.0_f32).max(60.0_f32) as i32;
        }
        if x < self.gap_until_x {
            solid = false;
        }

        let y = if solid {
            let xf = x as f32;
            let h = self.wave_amp_a * (xf * self.wave_freq_a + self.wave_phase_a).sin()
                + self.wave_amp_b * (xf * self.wave_freq_b + self.wave_phase_b).sin()
                + self.wave_amp_c * (xf * self.wave_freq_c + self.wave_phase_c).sin();

            // Convert height offset (positive => higher ground).
            let top = (GROUND_Y - h * 0.55).clamp(160.0, GROUND_Y - 8.0);
            top
        } else {
            // Value doesn't matter for gaps much, but keep it reasonable for visuals.
            GROUND_Y
        };

        // Determine tile type based on slope and sensible sequencing
        let tile = if solid {
            // Calculate slope from previous column
            let slope = self.last_terrain_y - y; // positive = rising (uphill to right)

            // Slope thresholds (in pixels per column)
            const SLOPE_STEEP_THRESHOLD: f32 = 3.0;
            const SLOPE_GENTLE_THRESHOLD: f32 = 0.8;

            // Minimum run length for slopes (prevents jittery tiles)
            const MIN_SLOPE_RUN: i32 = 8;

            // Determine next tile based on slope and current state
            match self.last_tile {
                SnowTile::Flat => {
                    if slope > SLOPE_STEEP_THRESHOLD {
                        // Starting steep uphill: use round transition
                        self.slope_run_length = 0;
                        SnowTile::SlopeLeftRound
                    } else if slope < -SLOPE_STEEP_THRESHOLD {
                        // Starting steep downhill: use round transition
                        self.slope_run_length = 0;
                        SnowTile::SlopeRightRound
                    } else {
                        // Stay flat for gentle slopes
                        SnowTile::Flat
                    }
                }
                SnowTile::SlopeLeftRound => {
                    // After round transition, go to proper slope
                    self.slope_run_length = 1;
                    SnowTile::SlopeLeft
                }
                SnowTile::SlopeLeft => {
                    self.slope_run_length += 1;
                    if slope < SLOPE_GENTLE_THRESHOLD || self.slope_run_length > 32 {
                        // Slope ending: transition back to flat
                        SnowTile::Flat
                    } else {
                        SnowTile::SlopeLeft
                    }
                }
                SnowTile::SlopeRightRound => {
                    // After round transition, go to proper slope
                    self.slope_run_length = 1;
                    SnowTile::SlopeRight
                }
                SnowTile::SlopeRight => {
                    self.slope_run_length += 1;
                    if slope > -SLOPE_GENTLE_THRESHOLD || self.slope_run_length > 32 {
                        // Slope ending: transition back to flat
                        SnowTile::Flat
                    } else {
                        SnowTile::SlopeRight
                    }
                }
                SnowTile::SlopeDown => {
                    // Legacy tile name, convert to appropriate tile
                    if slope < -SLOPE_GENTLE_THRESHOLD {
                        SnowTile::SlopeRight
                    } else {
                        SnowTile::Flat
                    }
                }
            }
        } else {
            // Gap: reset to flat for when terrain resumes
            SnowTile::Flat
        };

        // Update state for next column
        if solid {
            self.last_tile = tile;
            self.last_terrain_y = y;
        } else {
            self.last_tile = SnowTile::Flat;
        }

        self.terrain.push(TerrainCol { y, solid, tile });

        // Rocks (only on solid columns).
        if solid {
            // Visual size matched to the Kenney 64x74 stone tile,
            // scaled down to fit our gameplay hitbox.
            let rock_h = 26.0;
            let rock_w = rock_h * (TILE_W / TILE_H); // keep tile aspect ratio

            // Only place rocks when the ground isn't too high (keeps them on-screen).
            if y > GROUND_Y - 90.0 && self.score >= ROCK_SPAWN_MIN_SCORE {
                let spacing_ok = (x - self.last_rock_x) > 260;
                let rock_prob = (0.035 + difficulty * 0.02).min(0.14);
                if spacing_ok && rand::gen_range(0.0, 1.0) < rock_prob {
                    let rock_x = x as f32 - rock_w * 0.5;
                    let rock_y = y - rock_h;
                    if rock_y > -40.0 {
                        self.rocks.push(Rock {
                            x: rock_x,
                            y: rock_y,
                            w: rock_w,
                            h: rock_h,
                        });
                        self.last_rock_x = x;
                    }
                }
            }
        }
    }

    fn cull_behind_and_tidy(&mut self) {
        let min_keep_x = (self.cam_x.floor() - SCREEN_W * TERRAIN_BEHIND_SCREENS) as i32;
        let remove = min_keep_x - self.terrain_start_x;
        if remove > 0 {
            let remove_usize = remove as usize;
            if remove_usize < self.terrain.len() {
                self.terrain.drain(0..remove_usize);
                self.terrain_start_x = min_keep_x;
            } else {
                self.terrain.clear();
                self.terrain_start_x = min_keep_x;
            }
        }

        // Cull rocks behind the camera.
        let world_left = self.cam_x - 200.0;
        self.rocks.retain(|r| r.x + r.w >= world_left);
    }

    fn slope_delta_at_feet_center(&self, feet_center_x: f32) -> Option<f32> {
        let ix = feet_center_x.floor() as i32;
        let c0 = self.terrain_at(ix)?;
        let c1 = self.terrain_at(ix + 1)?;
        if !c0.solid || !c1.solid {
            return None;
        }
        Some(c1.y - c0.y)
    }

    fn slope_delta_at_feet(&self) -> Option<f32> {
        // Average slope between left and right parts of the player's footprint
        // to avoid "center column only" artifacts on terrain edges.
        let left = self.player.x + self.player.w * 0.25;
        let right = self.player.x + self.player.w * 0.75;

        let d1 = self.slope_delta_at_feet_center(left);
        let d2 = self.slope_delta_at_feet_center(right);

        match (d1, d2) {
            (Some(a), Some(b)) => Some((a + b) * 0.5),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        }
    }

    // Spectacle: trigger screen shake with given intensity
    fn trigger_shake(&mut self, intensity: f32) {
        self.shake_intensity = self.shake_intensity.max(intensity);
    }

    // Spectacle: update shake decay (call once per frame)
    fn update_shake(&mut self, dt: f32) {
        if self.shake_intensity > 0.0 {
            // Decay intensity
            self.shake_intensity -= SHAKE_DECAY * dt;
            if self.shake_intensity < 0.0 {
                self.shake_intensity = 0.0;
            }

            // Generate random offset based on intensity
            let angle = rand::gen_range(0.0, std::f32::consts::TAU);
            let mag = self.shake_intensity * SCREEN_W.min(SCREEN_H);
            self.shake_offset_x = angle.cos() * mag;
            self.shake_offset_y = angle.sin() * mag;
        } else {
            self.shake_offset_x = 0.0;
            self.shake_offset_y = 0.0;
        }
    }

    // Spectacle: trigger flash effect
    fn trigger_flash(&mut self, alpha: f32, color: Color) {
        self.flash_alpha = self.flash_alpha.max(alpha);
        self.flash_color = color;
    }

    // Spectacle: update flash decay
    fn update_flash(&mut self, dt: f32) {
        if self.flash_alpha > 0.0 {
            // Fade out flash
            self.flash_alpha -= (1.0 / FLASH_DURATION) * dt;
            if self.flash_alpha < 0.0 {
                self.flash_alpha = 0.0;
            }
        }
    }

    // Spectacle: trigger fade out (to black)
    fn trigger_fade_out(&mut self) {
        self.fade_state = FadeState::FadingOut;
    }

    // Spectacle: trigger fade in (from black)
    fn trigger_fade_in(&mut self) {
        self.fade_state = FadeState::FadingIn;
        self.fade_alpha = 1.0;
    }

    // Spectacle: update fade transition
    fn update_fade(&mut self, dt: f32) {
        match self.fade_state {
            FadeState::None => {
                self.fade_alpha = 0.0;
            }
            FadeState::FadingOut => {
                self.fade_alpha += FADE_SPEED * dt;
                if self.fade_alpha >= 1.0 {
                    self.fade_alpha = 1.0;
                    self.fade_state = FadeState::HoldBlack;
                }
            }
            FadeState::FadingIn => {
                self.fade_alpha -= FADE_SPEED * dt;
                if self.fade_alpha <= 0.0 {
                    self.fade_alpha = 0.0;
                    self.fade_state = FadeState::None;
                }
            }
            FadeState::HoldBlack => {
                // Hold for a moment before fading in
                // This state is manually exited by calling trigger_fade_in()
                self.fade_alpha = 1.0;
            }
        }
    }

    // Spectacle: update entrance animation (player drops from above)
    fn update_entrance(&mut self, dt: f32) {
        if !self.is_in_entrance {
            return;
        }

        self.entrance_timer += dt;

        // Find the ground at player's X position
        let player_center_x = self.player.x + self.player.w * 0.5;
        let ground_y = self
            .terrain_at(player_center_x.floor() as i32)
            .map(|c| c.y)
            .unwrap_or(GROUND_Y);
        let target_y = ground_y - self.player.h;

        // Bounce ease-out animation: y = start + (target - start) * ease
        let t = (self.entrance_timer / ENTRANCE_SLAM_DURATION).min(1.0);
        // Bounce easing: overshoot and settle
        let bounce_t = if t < 0.6 {
            // Fast drop
            t / 0.6
        } else {
            // Bounce back
            let bounce_t = (t - 0.6) / 0.4;
            1.0 + 0.15 * (1.0 - bounce_t) * (std::f32::consts::PI * bounce_t).sin()
        };

        self.player.y = ENTRANCE_START_Y + (target_y - ENTRANCE_START_Y) * bounce_t;

        // Check if we've landed (animation complete)
        if t >= 1.0 {
            self.is_in_entrance = false;
            self.player.y = target_y;
            self.player.on_ground = true;

            // Trigger landing shake
            self.trigger_shake(ENTRANCE_SHAKE_INTENSITY);

            // Spawn dust particles
            self.spawn_dust_particles(1.0);
        }
    }

    fn kill(&mut self) {
        if self.state == GameState::GameOver {
            return;
        }

        self.state = GameState::GameOver;

        // Update hi-score once at death.
        let final_score = self.score.floor() as i32;
        if final_score > self.hi_score {
            self.hi_score = final_score;
            unsafe {
                js_save_hi(self.hi_score);
            }
        }
    }

    pub fn update(&mut self) {
        let dt = get_frame_time().min(0.05);
        let dt_factor = dt * 60.0; // "frame equivalent" scaling
        self.time += dt;

        // Spectacle: update screen shake (runs every frame)
        self.update_shake(dt);

        // Spectacle: update flash decay
        self.update_flash(dt);

        // Spectacle: update fade transitions
        self.update_fade(dt);

        // Spectacle: update entrance animation
        self.update_entrance(dt);

        // Input shortcuts.
        let back_pressed = is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::Backspace);
        let enter_pressed = is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::KpEnter);

        match self.state {
            GameState::Playing => {
                if back_pressed {
                    self.state = GameState::Paused;
                    unsafe {
                        js_set_screen(2, 0, 0);
                    }
                    return;
                }

                // Keep UI in sync.
                unsafe {
                    js_set_screen(1, 0, 0);
                }

                // Keep terrain generated around the player for collisions/drawing.
                let view_right = (self.cam_x.floor() + SCREEN_W + 64.0) as i32;
                let player_ahead = (self.player.x + SCREEN_W * TERRAIN_AHEAD_SCREENS) as i32;
                let needed_x = player_ahead.max(view_right);
                self.ensure_generated_until(needed_x + 2);
                self.cull_behind_and_tidy();

                let jump_pressed = is_key_pressed(KeyCode::Up)
                    || is_key_pressed(KeyCode::Space)
                    || enter_pressed;

                let jump_held = is_key_down(KeyCode::Up)
                    || is_key_down(KeyCode::Space)
                    || is_key_down(KeyCode::Enter);

                // Spectacle: skip normal physics during entrance animation
                if self.is_in_entrance {
                    // Still update camera during entrance
                    let target_cam_x = self.player.x - SCREEN_W * 0.35;
                    self.cam_x += (target_cam_x - self.cam_x) * 0.12;
                    return;
                }

                if jump_pressed && self.player.on_ground {
                    self.player.vy = JUMP_FORCE_PER_FRAME;
                    self.player.on_ground = false;
                    self.player.backflip_happened = false; // fresh airtime
                }

                // Horizontal auto-run acceleration.
                self.player.vx = (self.player.vx + SPEED_INCREASE_PER_FRAME * dt_factor)
                    .min(MAX_SPEED_PER_FRAME)
                    .max(RUN_SPEED_PER_FRAME * 0.25);

                // Slope physics (only when grounded).
                if self.player.on_ground {
                    if let Some(delta_y) = self.slope_delta_at_feet() {
                        // If terrain rises to the right, delta_y > 0 => uphill drag.
                        // If terrain falls to the right, delta_y < 0 => downhill boost.
                        let dv = self.player.vx.abs() * delta_y.abs() * SLOPE_GAIN;
                        if delta_y < 0.0 {
                            self.player.vx += dv;
                        } else {
                            self.player.vx -= dv;
                        }
                        self.player.vx = self
                            .player
                            .vx
                            .clamp(RUN_SPEED_PER_FRAME * 0.2, MAX_SPEED_PER_FRAME);
                    }
                }

                // Integrate velocities.
                self.player.vx = self.player.vx.max(0.0);

                let prev_x = self.player.x;
                let dx = self.player.vx * dt_factor;
                self.player.x += dx;
                self.distance_travelled += dx;
                self.score += dx * self.combo * SCORE_DISTANCE_MULT;

                let prev_bottom = self.player.y + self.player.h;
                self.player.vy += GRAVITY_PER_FRAME * dt_factor;
                self.player.y += self.player.vy * dt_factor;

                // Terrain collision (land on solid columns, fall through gaps).
                let was_on_ground = self.player.on_ground;
                let vy_before_collision = self.player.vy;
                self.player.on_ground = false;
                if vy_before_collision >= -0.5 {
                    // Landing collision should only trigger when the player's
                    // bottom actually *crosses* the terrain top from above.
                    // Also, when moving right, exclude terrain columns that were
                    // not under the player's footprint in the previous frame.
                    // This prevents "teleporting up" when you hit a rising wall.
                    let bottom = self.player.y + self.player.h;
                    let prev_right_x = prev_x + self.player.w;

                    let left_x_world = self.player.x.floor() as i32;
                    let right_x_world = (self.player.x + self.player.w - 1.0).floor() as i32;

                    const TOP_CROSS_EPS: f32 = 2.8;
                    const TOP_PEN_EPS: f32 = 4.0;

                    let mut best_col_y: Option<f32> = None; // pick the highest crossed surface
                    for wx in left_x_world..=right_x_world {
                        // If the column starts after where the player's right edge
                        // was in the previous frame, it was likely a "wall entry" this frame.
                        if (wx as f32) >= prev_right_x {
                            continue;
                        }

                        let Some(col) = self.terrain_at(wx) else { continue };
                        if !col.solid {
                            continue;
                        }

                        let depth = bottom - col.y;

                        // Crossing condition (falling onto top).
                        let crossed = prev_bottom <= col.y + TOP_CROSS_EPS && bottom >= col.y;

                        // Discrete-step penetration: if we were already slightly above the
                        // surface last frame but ended up intersecting this frame,
                        // allow a landing correction.
                        let penetrating = depth >= 0.0
                            && prev_bottom > col.y
                            && (prev_bottom - col.y) <= TOP_PEN_EPS
                            && depth <= TOP_PEN_EPS;

                        if crossed || penetrating {
                            best_col_y = Some(match best_col_y {
                                Some(prev) => prev.min(col.y), // smaller y => higher surface
                                None => col.y,
                            });
                        }
                    }

                    if let Some(col_y) = best_col_y {
                        self.player.y = col_y - self.player.h;
                        self.player.vy = 0.0;
                        self.player.on_ground = true;
                    }
                }

                // Backflip rotation + landing combo logic.
                if !self.player.on_ground && jump_held {
                    self.player.angle += BACKFLIP_ROT_PER_FRAME * dt_factor;
                    self.player.backflip_happened = true;
                } else if self.player.on_ground {
                    // Dampen angle back towards "upright" when grounded.
                    // Faster snap back upright so rotation doesn't linger.
                    self.player.angle *= (0.5_f32).powf(dt_factor / 2.0);
                    if self.player.angle.abs() < 0.01 {
                        self.player.angle = 0.0;
                    }
                }

                // If we transitioned from air -> ground, treat it as a landing.
                if !was_on_ground && self.player.on_ground {
                    let impact = vy_before_collision.abs();
                    let clean = impact <= SCORE_CLEAN_LANDING_IMPACT_MAX;

                    if clean && self.player.backflip_happened {
                        self.combo += COMBO_INCREMENT;
                    } else if !clean {
                        self.combo = 1.0;
                    }

                    // Dust on landing (stronger for clean landings).
                    let dust_scale = if clean { 1.0 } else { 0.7 };
                    self.spawn_dust_particles(dust_scale);

                    // Landing consumes the backflip state.
                    self.player.backflip_happened = false;

                    // Rotation snap-back: clean landings still show a tiny tilt,
                    // rough landings fully reset orientation.
                    if clean {
                        self.player.angle *= 0.35;
                    } else {
                        self.player.angle = 0.0;
                    }
                }

                // Rock collision: kill on contact.
                for rock in &self.rocks {
                    // Fast rejection.
                    if rock.x > self.player.x + self.player.w + 80.0 {
                        break;
                    }
                    if rock.x + rock.w < self.player.x - 80.0 {
                        continue;
                    }

                    if self.player.x < rock.x + rock.w
                        && self.player.x + self.player.w > rock.x
                        && self.player.y < rock.y + rock.h
                        && self.player.y + self.player.h > rock.y
                    {
                        self.kill();
                        break;
                    }
                }

                // Fall death.
                if self.player.y > DEATH_Y {
                    self.kill();
                }

                // Camera follows player horizontally.
                let target_cam_x = self.player.x - SCREEN_W * 0.35;
                self.cam_x += (target_cam_x - self.cam_x) * 0.12;

                // Speed particles when the player is going fast.
                if self.player.vx >= SPEED_PARTICLE_VX_THRESHOLD {
                    // Spawn with a simple time gate to avoid over-spawning.
                    if self.time - self.last_speed_particle_t > (1.0 / 40.0) {
                        self.spawn_speed_particles();
                        self.last_speed_particle_t = self.time;
                    }
                }

                self.update_particles(dt_factor);

                // When running as WASM, update the HTML overlay HUD
                // (better typography than canvas text).
                unsafe {
                    js_set_hud(
                        self.score.floor() as i32,
                        self.combo,
                        self.hi_score,
                        self.distance_travelled.floor() as i32,
                    );
                }
            }
            GameState::Paused => {
                // Keep background/terrain as-is, but freeze physics.
                unsafe {
                    js_set_screen(2, 0, 0);
                }

                if back_pressed || enter_pressed {
                    self.state = GameState::Playing;
                }
            }
            GameState::GameOver => {
                unsafe {
                    js_set_screen(3, self.score.floor() as i32, self.hi_score);
                }

                if enter_pressed {
                    self.reset();
                }
            }
        }
    }

    pub fn draw(&self) {
        // Sky + ground background (no shake - parallax only)
        self.draw_background_parallax();

        // Apply shake offset to all game elements
        self.draw_terrain_with_shake();
        self.draw_rocks_with_shake();
        self.draw_particles_with_shake();
        self.draw_player_with_shake();

        self.draw_hud();

        // Canvas UI overlays are only for native builds.
        // Browser/WASM uses the HTML overlay so Orbitron typography works.
        #[cfg(not(target_arch = "wasm32"))]
        {
            if self.state == GameState::GameOver {
                draw_rectangle(
                    0.0,
                    0.0,
                    SCREEN_W,
                    SCREEN_H,
                    Color::new(0.0, 0.0, 0.0, 0.45),
                );
                draw_text(
                    "Game Over",
                    SCREEN_W * 0.5 - 105.0,
                    SCREEN_H * 0.35,
                    48.0,
                    WHITE,
                );

                // Display final score in center
                let score_i = self.score.floor() as i32;
                draw_text(
                    &format!("Score: {}", score_i),
                    SCREEN_W * 0.5 - 120.0,
                    SCREEN_H * 0.5,
                    36.0,
                    YELLOW,
                );

                // Show hi-score if beaten
                if score_i >= self.hi_score && self.hi_score > 0 {
                    draw_text(
                        &format!("New Best!"),
                        SCREEN_W * 0.5 - 90.0,
                        SCREEN_H * 0.5 + 40.0,
                        24.0,
                        Color::new(1.0, 0.8, 0.2, 1.0),
                    );
                } else if self.hi_score > 0 {
                    draw_text(
                        &format!("Best: {}", self.hi_score),
                        SCREEN_W * 0.5 - 100.0,
                        SCREEN_H * 0.5 + 40.0,
                        24.0,
                        LIGHTGRAY,
                    );
                }

                draw_text(
                    "Press Enter to Restart",
                    SCREEN_W * 0.5 - 250.0,
                    SCREEN_H * 0.5 + 80.0,
                    24.0,
                    LIGHTGRAY,
                );
            } else if self.state == GameState::Paused {
                draw_rectangle(
                    0.0,
                    0.0,
                    SCREEN_W,
                    SCREEN_H,
                    Color::new(0.0, 0.0, 0.0, 0.55),
                );
                draw_text(
                    "Paused",
                    SCREEN_W * 0.5 - 70.0,
                    SCREEN_H * 0.5 - 20.0,
                    48.0,
                    LIGHTGRAY,
                );
                draw_text(
                    "Press Enter to Resume",
                    SCREEN_W * 0.5 - 170.0,
                    SCREEN_H * 0.5 + 40.0,
                    24.0,
                    WHITE,
                );
            }
        }

        // Spectacle: draw flash overlay (last so it's on top of everything)
        self.draw_flash();

        // Spectacle: draw fade overlay (on top of flash)
        self.draw_fade();
    }

    fn draw_background_parallax(&self) {
        // Draw the Pixel Skies background (demo06 - sunset/dusk theme)
        // Apply subtle parallax by scrolling the background slower than the camera
        let render_cam_x = self.cam_x.floor();
        let bg_parallax = 0.05; // background moves at 5% of camera speed

        // Get background dimensions
        let bg_w = self.bg_tex.width();
        let bg_h = self.bg_tex.height();

        // Calculate scroll offset
        let scroll_x = (render_cam_x * bg_parallax) % bg_w;

        // Draw two copies for seamless looping
        // Scale to fit screen height while maintaining aspect ratio
        let scale = SCREEN_H / bg_h;
        let scaled_w = bg_w * scale;

        for i in -1..=1 {
            let x = i as f32 * scaled_w - scroll_x;
            draw_texture_ex(
                &self.bg_tex,
                x,
                0.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(scaled_w, SCREEN_H)),
                    ..Default::default()
                },
            );
        }
    }

    fn draw_terrain_with_shake(&self) {
        let render_cam_x = self.cam_x.floor();
        let shake_x = self.shake_offset_x;
        let shake_y = self.shake_offset_y;

        // Draw terrain as tiles (64px wide) for the snow tile look
        let tile_w = TILE_W as i32;
        let tile_left = ((render_cam_x - tile_w as f32) as i32 / tile_w) * tile_w;
        let tile_right = tile_left + (SCREEN_W as i32 / tile_w + 3) * tile_w;

        for tile_x in (tile_left..=tile_right).step_by(tile_w as usize) {
            // Sample the terrain at the left and right edges of this tile
            let col_left = match self.terrain_at(tile_x) {
                Some(c) => c,
                None => continue,
            };

            // Skip gaps
            if !col_left.solid {
                continue;
            }

            // Get the tile type for this position
            let tile = col_left.tile;

            // Determine source rectangle based on tile type
            let src_rect = match tile {
                SnowTile::Flat => Rect::new(SNOW_FLAT_X, SNOW_FLAT_Y, TILE_W, TILE_H),
                SnowTile::SlopeDown => Rect::new(SNOW_SLOPE_DOWN_X, SNOW_SLOPE_DOWN_Y, TILE_W, TILE_H),
                SnowTile::SlopeLeft => Rect::new(SNOW_SLOPE_LEFT_X, SNOW_SLOPE_LEFT_Y, TILE_W, TILE_H - 2.0),
                SnowTile::SlopeLeftRound => Rect::new(SNOW_SLOPE_LEFT_ROUND_X, SNOW_SLOPE_LEFT_ROUND_Y, TILE_W, TILE_H),
                SnowTile::SlopeRight => Rect::new(SNOW_SLOPE_RIGHT_X, SNOW_SLOPE_RIGHT_Y, TILE_W, TILE_H - 2.0),
                SnowTile::SlopeRightRound => Rect::new(SNOW_SLOPE_RIGHT_ROUND_X, SNOW_SLOPE_RIGHT_ROUND_Y, TILE_W, TILE_H),
            };

            // Screen position with shake offset
            let sx = tile_x as f32 - render_cam_x + shake_x;

            // Get the terrain Y at this position
            let terrain_y = col_left.y;

            // For slope tiles, we need to adjust position because the sprite
            // has the slope built-in. The tile's "base" is at bottom-left.
            // For flat tiles, draw so top aligns with terrain_y.
            // For slopes, we adjust based on the slope type.

            let draw_y = match tile {
                SnowTile::Flat => terrain_y,
                SnowTile::SlopeLeft | SnowTile::SlopeLeftRound => {
                    // Slope up-left: the left side is the highest point
                    terrain_y
                }
                SnowTile::SlopeRight | SnowTile::SlopeRightRound => {
                    // Slope up-right: the left side is lower
                    terrain_y - TILE_H + 10.0 // approximate adjustment
                }
                SnowTile::SlopeDown => {
                    // Slope down-right
                    terrain_y
                }
            };

            // Draw the tile with shake offset
            draw_texture_ex(
                &self.player_tex,
                sx,
                draw_y + shake_y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(TILE_W, TILE_H)),
                    source: Some(src_rect),
                    ..Default::default()
                },
            );
        }
    }

    fn draw_rocks_with_shake(&self) {
        let render_cam_x = self.cam_x.floor();
        let shake_x = self.shake_offset_x;
        let shake_y = self.shake_offset_y;

        for r in &self.rocks {
            // Cull by x range (screen coords).
            let sx = r.x - render_cam_x;
            if sx > SCREEN_W + 80.0 || sx + r.w < -80.0 {
                continue;
            }

            // Draw stone block from the Kenney atlas with shake offset.
            let dest_size = vec2(r.w, r.h);
            let src_rect = Rect::new(STONE_SRC_X, STONE_SRC_Y, TILE_W, TILE_H);
            draw_texture_ex(
                &self.player_tex,
                sx + shake_x,
                r.y + shake_y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(dest_size),
                    source: Some(src_rect),
                    ..Default::default()
                },
            );

            draw_rectangle_lines(
                sx + shake_x,
                r.y + shake_y,
                r.w,
                r.h,
                2.0,
                Color::new(0.1, 0.1, 0.15, 1.0),
            );
        }
    }

    fn draw_particles_with_shake(&self) {
        let render_cam_x = self.cam_x.floor();
        let shake_x = self.shake_offset_x;
        let shake_y = self.shake_offset_y;

        for p in &self.particles {
            let sx = p.x - render_cam_x;
            if sx < -80.0 || sx > SCREEN_W + 80.0 {
                continue;
            }
            let sy = p.y;
            if sy < -80.0 || sy > SCREEN_H + 80.0 {
                continue;
            }
            draw_circle(sx + shake_x, sy + shake_y, p.size, p.color);
        }
    }

    fn draw_player_with_shake(&self) {
        let render_cam_x = self.cam_x.floor();
        let shake_x = self.shake_offset_x;
        let shake_y = self.shake_offset_y;

        let px = (self.player.x - render_cam_x).floor();
        let py = self.player.y.floor();

        let body_col = if self.player.on_ground {
            Color::new(0.95, 0.95, 0.98, 1.0)
        } else {
            Color::new(0.9, 0.92, 0.98, 1.0)
        };

        // Draw sprite from atlas; keep sprite centered in hitbox.
        // (The source sprite is 16x25, but the hitbox is 42x42.)
        let dest_h = self.player.h;
        let dest_w = PLAYER_SPRITE_SRC_W * (dest_h / PLAYER_SPRITE_SRC_H);
        let dest_x = px + (self.player.w - dest_w) * 0.5;
        let dest_y = py;

        draw_texture_ex(
            &self.player_tex,
            dest_x + shake_x,
            dest_y + shake_y,
            body_col,
            DrawTextureParams {
                dest_size: Some(vec2(dest_w, dest_h)),
                source: Some(Rect::new(
                    PLAYER_SPRITE_SRC_X,
                    PLAYER_SPRITE_SRC_Y,
                    PLAYER_SPRITE_SRC_W,
                    PLAYER_SPRITE_SRC_H,
                )),
                rotation: self.player.angle,
                ..Default::default()
            },
        );

        // Simple outline to keep readability even with transparent sprite pixels.
        draw_rectangle_lines(
            px + shake_x,
            py + shake_y,
            self.player.w,
            self.player.h,
            2.0,
            Color::new(0.1, 0.1, 0.15, 1.0),
        );
    }

    fn draw_rocks(&self) {
        let render_cam_x = self.cam_x.floor();

        for r in &self.rocks {
            // Cull by x range (screen coords).
            let sx = r.x - render_cam_x;
            if sx > SCREEN_W + 80.0 || sx + r.w < -80.0 {
                continue;
            }

            // Draw stone block from the Kenney atlas.
            let dest_size = vec2(r.w, r.h);
            let src_rect = Rect::new(STONE_SRC_X, STONE_SRC_Y, TILE_W, TILE_H);
            draw_texture_ex(
                &self.player_tex,
                sx,
                r.y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(dest_size),
                    source: Some(src_rect),
                    ..Default::default()
                },
            );

            draw_rectangle_lines(
                sx,
                r.y,
                r.w,
                r.h,
                2.0,
                Color::new(0.1, 0.1, 0.15, 1.0),
            );
        }
    }

    fn draw_player(&self) {
        let render_cam_x = self.cam_x.floor();
        let px = (self.player.x - render_cam_x).floor();
        let py = self.player.y.floor();

        let body_col = if self.player.on_ground {
            Color::new(0.95, 0.95, 0.98, 1.0)
        } else {
            Color::new(0.9, 0.92, 0.98, 1.0)
        };

        // Draw sprite from atlas; keep sprite centered in hitbox.
        // (The source sprite is 16x25, but the hitbox is 42x42.)
        let dest_h = self.player.h;
        let dest_w = PLAYER_SPRITE_SRC_W * (dest_h / PLAYER_SPRITE_SRC_H);
        let dest_x = px + (self.player.w - dest_w) * 0.5;
        let dest_y = py;

        draw_texture_ex(
            &self.player_tex,
            dest_x,
            dest_y,
            body_col,
            DrawTextureParams {
                dest_size: Some(vec2(dest_w, dest_h)),
                source: Some(Rect::new(
                    PLAYER_SPRITE_SRC_X,
                    PLAYER_SPRITE_SRC_Y,
                    PLAYER_SPRITE_SRC_W,
                    PLAYER_SPRITE_SRC_H,
                )),
                rotation: self.player.angle,
                ..Default::default()
            },
        );

        // Simple outline to keep readability even with transparent sprite pixels.
        draw_rectangle_lines(
            px,
            py,
            self.player.w,
            self.player.h,
            2.0,
            Color::new(0.1, 0.1, 0.15, 1.0),
        );
    }

    fn draw_particles(&self) {
        let render_cam_x = self.cam_x.floor();
        for p in &self.particles {
            let sx = p.x - render_cam_x;
            if sx < -80.0 || sx > SCREEN_W + 80.0 {
                continue;
            }
            let sy = p.y;
            if sy < -80.0 || sy > SCREEN_H + 80.0 {
                continue;
            }
            draw_circle(sx, sy, p.size, p.color);
        }
    }

    fn update_particles(&mut self, dt_factor: f32) {
        for p in &mut self.particles {
            p.x += p.vx * dt_factor;
            p.y += p.vy * dt_factor;
            // A tiny gravity so dust/sparks arc.
            p.vy += 0.08 * dt_factor;
            p.life -= dt_factor;
        }

        self.particles.retain(|p| p.life > 0.0);
    }

    fn spawn_dust_particles(&mut self, scale: f32) {
        // Spawn around the feet with quick outward burst.
        let base_x = self.player.x + self.player.w * 0.5;
        let base_y = self.player.y + self.player.h;

        let (r, g, b) = if scale > 0.95 {
            (210.0, 210.0, 220.0)
        } else {
            (160.0, 160.0, 175.0)
        };

        for _ in 0..DUST_SPAWN_ON_LAND_COUNT {
            let ang: f32 = rand::gen_range(-1.6, 1.6);
            let spd = rand::gen_range(0.5, 2.4) * scale;
            let vx = ang.cos() * spd * 2.0;
            let vy = ang.sin() * spd * 2.0 - rand::gen_range(0.0, 1.2);
            let life = rand::gen_range(12.0, 20.0);
            self.particles.push(Particle {
                x: base_x + rand::gen_range(-10.0, 10.0),
                y: base_y + rand::gen_range(-2.0, 2.0),
                vx,
                vy,
                life,
                max_life: life,
                size: rand::gen_range(1.0, 3.0) * scale,
                color: Color::new(r / 255.0, g / 255.0, b / 255.0, 0.7),
            });
        }
    }

    fn spawn_speed_particles(&mut self) {
        // Spawn behind the player to visualize speed.
        let _cam_x = self.cam_x.floor();
        let world_x = self.player.x + self.player.w * 0.5;
        let world_y = self.player.y + self.player.h * 0.6;

        let streak_len = 18;
        for i in 0..streak_len {
            if i % 3 != 0 {
                continue;
            }
            let offset = i as f32 * 2.5;
            let t = i as f32 / streak_len as f32;
            let vx = -rand::gen_range(2.0, 4.0) - self.player.vx * 0.08 * (1.0 - t);
            let vy = rand::gen_range(-1.0, 1.0);
            let life = rand::gen_range(10.0, 16.0);
            self.particles.push(Particle {
                x: world_x - offset + rand::gen_range(-6.0, 6.0),
                y: world_y + rand::gen_range(-10.0, 8.0),
                vx,
                vy,
                life,
                max_life: life,
                size: rand::gen_range(1.0, 2.6),
                color: Color::new(0.2, 0.7, 1.0, 0.55),
            });
        }

        // Basic cap to keep runtime stable.
        if self.particles.len() > 450 {
            self.particles
                .retain(|p| p.life > 0.25 * p.max_life);
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn draw_hud(&self) {}

    #[cfg(not(target_arch = "wasm32"))]
    fn draw_hud(&self) {
        let score_i = self.score.floor() as i32;
        let combo = self.combo;
        let dist_i = self.distance_travelled.floor() as i32;

        // Native fallback HUD: keep it readable and a bit more styled.
        draw_text(
            &format!("Score: {}", score_i),
            16.0,
            24.0,
            26.0,
            WHITE,
        );
        draw_text(
            &format!("Combo x{:.1}", combo),
            SCREEN_W * 0.5 - 120.0,
            24.0,
            26.0,
            YELLOW,
        );
        draw_text(
            &format!("Hi: {}", self.hi_score),
            SCREEN_W - 170.0,
            24.0,
            26.0,
            LIGHTGRAY,
        );
        draw_text(
            &format!("Dist: {}", dist_i),
            16.0,
            56.0,
            22.0,
            Color::new(0.8, 0.9, 0.95, 1.0),
        );
    }

    // Spectacle: draw flash overlay
    fn draw_flash(&self) {
        if self.flash_alpha > 0.01 {
            draw_rectangle(
                0.0,
                0.0,
                SCREEN_W,
                SCREEN_H,
                Color::new(
                    self.flash_color.r,
                    self.flash_color.g,
                    self.flash_color.b,
                    self.flash_alpha,
                ),
            );
        }
    }

    // Spectacle: draw fade overlay
    fn draw_fade(&self) {
        if self.fade_alpha > 0.01 {
            draw_rectangle(
                0.0,
                0.0,
                SCREEN_W,
                SCREEN_H,
                Color::new(0.0, 0.0, 0.0, self.fade_alpha),
            );
        }
    }
}

