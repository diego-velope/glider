use macroquad::prelude::*;

mod game;

fn window_conf() -> Conf {
    Conf {
        window_title: "Glider - Rust".to_owned(),
        window_width: 1280,
        window_height: 720,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let player_tex_res = load_texture("assets/kenney_block-pack/Spritesheet/blockPack_spritesheet.png").await;
    let bg_tex_res = load_texture("assets/pixel_skies/pixel_skies_1920x1080/demo06_PixelSky_1920x1080.png").await;
    let font_res = load_ttf_font("assets/Orbitron-Bold.ttf").await;

    if player_tex_res.is_err() || bg_tex_res.is_err() || font_res.is_err() {
        loop {
            clear_background(BLACK);
            
            let mut y = 50.0;
            draw_text("FAILED TO LOAD ASSETS", 30.0, y, 50.0, RED);
            y += 50.0;
            
            draw_text("The game cannot start because some files are missing.", 30.0, y, 24.0, WHITE);
            y += 40.0;

            if player_tex_res.is_err() {
                draw_text("- Missing: assets/kenney_block-pack/Spritesheet/blockPack_spritesheet.png", 30.0, y, 20.0, LIGHTGRAY);
                y += 30.0;
            }
            if bg_tex_res.is_err() {
                draw_text("- Missing: assets/pixel_skies/pixel_skies_1920x1080/demo06_PixelSky_1920x1080.png", 30.0, y, 20.0, LIGHTGRAY);
                y += 30.0;
            }
            if font_res.is_err() {
                draw_text("- Missing: assets/Orbitron-Bold.ttf", 30.0, y, 20.0, LIGHTGRAY);
                y += 30.0;
            }
            
            y += 20.0;
            draw_text("Check your deployed build (e.g. GitHub Pages) ensuring you uploaded the 'assets' directory", 30.0, y, 20.0, YELLOW);
            y += 30.0;
            draw_text("and that filename cases match exactly (Linux servers are case-sensitive).", 30.0, y, 20.0, YELLOW);
            
            next_frame().await;
        }
    }

    // Load the player sprite sheet atlas.
    let player_tex = player_tex_res.unwrap();
    player_tex.set_filter(FilterMode::Nearest);

    // Load the Pixel Skies background (demo06 - sunset/dusk theme).
    let bg_tex = bg_tex_res.unwrap();
    bg_tex.set_filter(FilterMode::Nearest);

    let ui_font = font_res.unwrap();

    let mut game = game::Game::new(player_tex, bg_tex, ui_font);

    loop {
        game.update();
        game.draw();
        next_frame().await
    }
}

