use crate::{game::Game, gfx::ChunkTables};
use glfw::{CursorMode, PWindow};

pub fn handle_input_actions(gamestate: &mut Game) {
    gamestate.toggle_backface();
    gamestate.toggle_hud();
    gamestate.pause();
    if !gamestate.paused || gamestate.display_block_menu {
        gamestate.player.hotbar.scroll(gamestate.get_scroll_state());
    }
}

pub fn rotate_player(gamestate: &mut Game, sensitivity: f32, window: &PWindow) {
    if gamestate.is_paused() {
        return;
    }

    if window.get_cursor_mode() != CursorMode::Disabled {
        return;
    }

    if !window.is_focused() {
        return;
    }

    gamestate.rotate_player(sensitivity);
}

pub fn update_game(gamestate: &mut Game, chunktables: &mut ChunkTables, dt: f32) {
    if gamestate.is_paused() {
        return;
    }

    //Update gameobjects
    gamestate.update_player(dt);
    //Destroy and place blocks
    gamestate.build(chunktables);
    gamestate.update_build_cooldown(dt);
    //Update hand animation
    gamestate.update_hand_animation(dt);
    //Update blocks
    gamestate.world.update_sim_range(1);
    gamestate.world.update_blocks(dt, chunktables, 1);
    gamestate.world.rand_block_update(dt, Some(chunktables), 1);

    //Generate new chunks
    gamestate.world.clean_cache();
    gamestate
        .world
        .generate_more(gamestate.player.position, chunktables);
    chunktables.update_tables(gamestate);
}
