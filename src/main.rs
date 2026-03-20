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
    // Load the player sprite sheet atlas.
    let player_tex: Texture2D =
        load_texture("assets/kenney_block-pack/Spritesheet/blockPack_spritesheet.png")
            .await
            .unwrap();
    player_tex.set_filter(FilterMode::Nearest);

    // Load the Pixel Skies background (demo06 - sunset/dusk theme).
    let bg_tex: Texture2D =
        load_texture("assets/pixel_skies/pixel_skies_1920x1080/demo06_PixelSky_1920x1080.png")
            .await
            .unwrap();
    bg_tex.set_filter(FilterMode::Nearest);

    let mut game = game::Game::new(player_tex, bg_tex);

    loop {
        game.update();
        game.draw();
        next_frame().await
    }
}

