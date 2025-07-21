use crate::{game::Game, gfx::ChunkTables, voxel::world::get_simulation_dist};
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
    let is_dead = gamestate.player.is_dead();
    gamestate.update_player(dt);
    //Just died
    if !is_dead && gamestate.player.is_dead() {
        gamestate
            .player
            .drop_inventory(&mut gamestate.entities.dropped_items);
    }
    gamestate
        .entities
        .update(dt, &gamestate.world, &mut gamestate.player);
    //Destroy and place blocks
    gamestate.build(chunktables, dt);
    gamestate.update_build_cooldown(dt);
    //Update hand animation
    gamestate.update_hand_animation(dt);
    //Update blocks
    let sim_range = get_simulation_dist(&gamestate.world);
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
