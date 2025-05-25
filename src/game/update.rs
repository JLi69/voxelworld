use super::inventory::Item;
use super::{Game, GameMode, KeyState};
use crate::gfx::{self, ChunkTables};
use crate::voxel::build::destroy_block_suffocating;
use crate::voxel::{destroy_block, place_block};
use glfw::{Key, MouseButtonLeft, MouseButtonRight};

const BUILD_COOLDOWN: f32 = 0.15;

const HOTBAR_KEYS: [Key; 9] = [
    Key::Num1,
    Key::Num2,
    Key::Num3,
    Key::Num4,
    Key::Num5,
    Key::Num6,
    Key::Num7,
    Key::Num8,
    Key::Num9,
];

impl Game {
    fn rotate_item(&mut self) {
        //Rotate the block in the player's hand
        if self.get_key_state(Key::R) == KeyState::JustPressed {
            if let Item::BlockItem(b, amt) = self.player.hotbar.get_selected() {
                match b.shape() {
                    1 => {
                        let mut rotated_block = b;
                        if rotated_block.orientation() == 0 {
                            rotated_block.set_orientation(2);
                        } else if rotated_block.orientation() != 0 {
                            rotated_block.set_orientation(0);
                        }
                        let new_item = Item::BlockItem(rotated_block, amt);
                        self.player.hotbar.set_selected(new_item);
                    }
                    2..=4 => {
                        let mut stair_block = b;
                        let shape = b.shape();
                        if shape == 4 {
                            stair_block.set_shape(2);
                            stair_block.set_orientation(2);
                        } else if shape == 3 {
                            stair_block.set_shape(4);
                            stair_block.set_orientation(4);
                        } else if shape == 2 {
                            stair_block.set_shape(3);
                            stair_block.set_orientation(4);
                        }
                        let new_item = Item::BlockItem(stair_block, amt);
                        self.player.hotbar.set_selected(new_item);
                    }
                    _ => {}
                }
            }
        }
    }

    pub fn rotate_player(&mut self, sensitivity: f32) {
        let (dmousex, dmousey) = self.get_mouse_diff();
        //Rotate camera
        self.cam.rotate(dmousex, dmousey, sensitivity);
    }

    //Update player and camera
    pub fn update_player(&mut self, dt: f32) {
        //Set rotation of player
        self.player.rotation = self.cam.yaw;
        //Update player
        self.player.update(dt, &self.world);
        //Set position of camera
        self.cam.position = self.player.position + self.player.cam_offset();
        //Move player
        let lshift = self.get_key_state(Key::LeftShift);
        let rshift = self.get_key_state(Key::RightShift);
        self.player.sprint(lshift);
        self.player.sprint_or(rshift);
        let lctrl = self.get_key_state(Key::LeftControl);
        let rctrl = self.get_key_state(Key::RightShift);
        self.player.crouch(lctrl);
        self.player.crouch_or(rctrl);
        self.player.set_speed();
        let w = self.get_key_state(Key::W);
        let s = self.get_key_state(Key::S);
        let a = self.get_key_state(Key::A);
        let d = self.get_key_state(Key::D);
        self.player.strafe(a, d);
        self.player.move_forward(w, s);
        //Jump or climb
        let space = self.get_key_state(Key::Space);
        if !self.player.climbing(&self.world) {
            self.player.jump(space);
        } else {
            self.player.climb(space, lctrl, &self.world)
        }
        //Swim
        self.player.swim(space, &self.world);
        //Select items from the hotbar
        for (i, key) in HOTBAR_KEYS.iter().enumerate() {
            let keystate = self.get_key_state(*key);
            self.player.select_hotbar_item(keystate, i);
        }
        //Rotate current item in the hotbar (if it is rotatable
        self.rotate_item();
        //Drop item
        if self.get_key_state(Key::Q) == KeyState::JustPressed {
            self.player.hotbar.set_selected(Item::EmptyItem);
        }

        match self.game_mode() {
            GameMode::Survival => self.player.update_survival(dt),
            GameMode::Creative => self.player.update_creative(dt),
        }
    }

    pub fn update_build_cooldown(&mut self, dt: f32) {
        self.build_cooldown -= dt;
        self.destroy_cooldown -= dt;
    }

    pub fn update_display_debug(&mut self) {
        if self.get_key_state(Key::F3) == KeyState::JustPressed {
            self.display_debug = !self.display_debug;
        }
    }

    //Place and destroy blocks
    pub fn build(&mut self, chunktables: &mut ChunkTables) {
        //Destroy blocks
        let pos = self.cam.position;
        let dir = self.cam.forward();
        if !self.get_mouse_state(MouseButtonLeft).is_held() {
            self.destroy_cooldown = 0.0;
        }

        let stuck = self.player.get_head_stuck_block(&self.world);
        if self.get_mouse_state(MouseButtonLeft).is_held()
            && self.destroy_cooldown <= 0.0
            && stuck.is_none()
        {
            let destroyed = destroy_block(pos, dir, &mut self.world);
            let update_mesh = self.world.update_single_block_light(destroyed);
            gfx::update_chunk_vaos(chunktables, destroyed, &self.world);
            for (x, y, z) in update_mesh {
                chunktables.update_table(&self.world, x, y, z);
            }
            if destroyed.is_some() {
                self.destroy_cooldown = BUILD_COOLDOWN;
            } else {
                self.destroy_cooldown = 0.0;
            }
        } else if self.get_mouse_state(MouseButtonLeft).is_held() && self.destroy_cooldown <= 0.0 {
            //If the player is trapped in a block, then they can only break
            //the block that is currently trapping them
            let destroyed = destroy_block_suffocating(stuck, &mut self.world);
            let update_mesh = self.world.update_single_block_light(destroyed);
            gfx::update_chunk_vaos(chunktables, destroyed, &self.world);
            for (x, y, z) in update_mesh {
                chunktables.update_table(&self.world, x, y, z);
            }
            if destroyed.is_some() {
                self.destroy_cooldown = BUILD_COOLDOWN;
            } else {
                self.destroy_cooldown = 0.0;
            }
        }

        //Place blocks
        if !self.get_mouse_state(MouseButtonRight).is_held() {
            self.build_cooldown = 0.0;
        }

        if self.get_mouse_state(MouseButtonRight).is_held()
            && self.build_cooldown <= 0.0
            && stuck.is_none()
        {
            let placed = place_block(pos, dir, &mut self.world, &self.player);
            let update_mesh = self.world.update_single_block_light(placed);
            gfx::update_chunk_vaos(chunktables, placed, &self.world);
            for (x, y, z) in update_mesh {
                chunktables.update_table(&self.world, x, y, z);
            }
            if placed.is_some() {
                self.hand_animation = 0.1;
                self.build_cooldown = BUILD_COOLDOWN;
            } else {
                self.build_cooldown = 0.0;
            }
        }
    }

    //Handle pausing
    pub fn pause(&mut self) {
        if self.get_key_state(Key::Escape) == KeyState::JustPressed {
            //Escape out of the block menu
            if self.display_block_menu {
                self.display_block_menu = false;
                self.paused = false;
                return;
            }

            //Pause the game
            self.paused = !self.paused;
        }

        //Toggle the block menu with Tab (Note: the block menu pauses the game)
        //Only enable block menu in creative mode
        if self.game_mode() == GameMode::Survival {
            self.display_block_menu = false;
            return;
        }

        if self.get_key_state(Key::Tab) == KeyState::JustPressed {
            self.paused = !self.paused;
            self.display_block_menu = !self.display_block_menu;
        }
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    pub fn get_display_block_menu(&self) -> bool {
        self.display_block_menu
    }

    pub fn get_hand_animation(&self) -> f32 {
        self.hand_animation
    }

    //Toggle hud
    pub fn toggle_hud(&mut self) {
        if self.get_key_state(Key::F1) == KeyState::JustPressed {
            self.display_hud = !self.display_hud;
        }
    }

    //Toggle backface
    //For debug purposes
    pub fn toggle_backface(&mut self) {
        if self.game_mode() == GameMode::Survival {
            self.invert_backface_culling = false;
            return;
        }

        if self.get_key_state(Key::F12) == KeyState::JustPressed {
            self.invert_backface_culling = !self.invert_backface_culling;
        }
    }

    //Update hand animation
    pub fn update_hand_animation(&mut self, dt: f32) {
        if self.get_mouse_state(MouseButtonLeft).is_held() {
            self.hand_animation += dt * 3.0;
            self.hand_animation = self.hand_animation.fract();
        } else {
            if self.hand_animation > 0.0 {
                self.hand_animation += dt * 3.0;
            }

            if self.hand_animation > 1.0 {
                self.hand_animation = 0.0;
            }
        }
    }
}
