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
    let sim_range = (gamestate.world.get_range() / 2).min(7);
    gamestate.world.update_sim_range(sim_range);
    gamestate.world.update_blocks(dt, chunktables, sim_range);
    gamestate.world.rand_block_update(dt, Some(chunktables), sim_range);
    //Update day night cycle
    gamestate.world.update_daynight(dt);

    //Generate new chunks
    gamestate.world.clean_cache();
    gamestate
        .world
        .generate_more(gamestate.player.position, chunktables);
    chunktables.update_tables(gamestate);
}
