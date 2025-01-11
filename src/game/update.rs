use super::{Game, KeyState};
use crate::gfx::{self, ChunkTables};
use crate::voxel::build::destroy_block_suffocating;
use crate::voxel::{destroy_block, place_block};
use glfw::{CursorMode, Key};
use glfw::{MouseButtonLeft, MouseButtonRight};

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
    //Update player and camera
    pub fn update_player(&mut self, dt: f32, cursormode: CursorMode) {
        if cursormode == CursorMode::Disabled {
            let (dmousex, dmousey) = self.get_mouse_diff();
            //Rotate camera
            self.cam.rotate(dmousex, dmousey, 0.06);
        }

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
        let space = self.get_key_state(Key::Space);
        //Jump
        self.player.jump(space);
        //Swim
        self.player.swim(space, &self.world);
        //Select items from the hotbar
        for (i, key) in HOTBAR_KEYS.iter().enumerate() {
            let keystate = self.get_key_state(*key);
            self.player.select_hotbar_item(keystate, i);
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

        let suffocating = self.player.suffocating(&self.world);
        if self.get_mouse_state(MouseButtonLeft).is_held() 
            && self.destroy_cooldown <= 0.0
            && !suffocating {
            let destroyed = destroy_block(pos, dir, &mut self.world);
            gfx::update_chunk_vaos(chunktables, destroyed, &self.world);
            if destroyed.is_some() {
                self.destroy_cooldown = BUILD_COOLDOWN;
            } else {
                self.destroy_cooldown = 0.0;
            }
        } else if self.get_mouse_state(MouseButtonLeft).is_held() && self.destroy_cooldown <= 0.0 { 
            //If the player is trapped in a block, then they can only break
            //the block that is currently trapping them
            let destroyed = destroy_block_suffocating(pos, &mut self.world);
            gfx::update_chunk_vaos(chunktables, destroyed, &self.world);
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
            && !suffocating
        {
            let placed = place_block(pos, dir, &mut self.world, &self.player);
            gfx::update_chunk_vaos(chunktables, placed, &self.world);
            if placed.is_some() {
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

    //Toggle hud
    pub fn toggle_hud(&mut self) {
        if self.get_key_state(Key::F1) == KeyState::JustPressed {
            self.display_hud = !self.display_hud;
        }
    }
}
