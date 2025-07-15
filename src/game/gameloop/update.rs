use crate::{game::Game, gfx::ChunkTables};
use glfw::{CursorMode, PWindow};

fn handle_hotbar_scroll(gamestate: &mut Game) {
    if gamestate.player.is_dead() {
        return;
    }

    if gamestate.paused && !gamestate.display_block_menu {
        return;
    }

    let prev_selected = gamestate.player.hotbar.selected;
    gamestate.player.hotbar.scroll(gamestate.get_scroll_state());
    let current_selected = gamestate.player.hotbar.selected;

    //Reset the break timer if the player switches items
    if prev_selected != current_selected {
        gamestate.player.break_timer = 0.0;
    }
}

pub fn handle_input_actions(gamestate: &mut Game) {
    gamestate.toggle_backface();
    gamestate.toggle_hud();
    gamestate.pause();
    handle_hotbar_scroll(gamestate);
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

    //Force load any chunks that are close enough to the player
    gamestate.world.force_load();

    //Update gameobjects
    gamestate.update_player(dt);
    //Destroy and place blocks
    gamestate.build(chunktables, dt);
    gamestate.update_build_cooldown(dt);
    //Update hand animation
    gamestate.update_hand_animation(dt);
    //Update blocks
    let sim_range = (gamestate.world.get_range() / 2 + 1)
        .min(7)
        .min(gamestate.world.get_range() - 1);
    gamestate.world.update_sim_range(sim_range);
    gamestate.world.update_blocks(dt, chunktables, sim_range);
    gamestate
        .world
        .rand_block_update(dt, Some(chunktables), sim_range);
    //Update day night cycle
    gamestate.world.update_daynight(dt);

    //Generate new chunks
    gamestate.world.clean_cache();
    gamestate
        .world
        .update_generation_queue(gamestate.player.position);
    gamestate.world.load_from_queue(0.01);
    gamestate.world.update_chunktables(chunktables);
    chunktables.update_tables(gamestate);
}
