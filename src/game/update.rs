use super::inventory::tools::ToolType;
use super::inventory::{item_to_string, remove_amt_item, Item};
use super::player::{DEFAULT_MAX_HEALTH, PLAYER_HEIGHT};
use super::{Game, GameMode, KeyState};
use crate::gfx::{self, ChunkTables};
use crate::voxel::block_info::get_drop;
use crate::voxel::build::{destroy_block_suffocating, interact_with_block};
use crate::voxel::world::block_update::break_ice;
use crate::voxel::{self, destroy_block, place_block, Block, World, EMPTY_BLOCK, FULL_BLOCK};
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
            if let Item::Block(b, amt) = self.player.hotbar.get_selected() {
                match b.shape() {
                    1 => {
                        let mut rotated_block = b;
                        if rotated_block.orientation() == 0 {
                            rotated_block.set_orientation(2);
                        } else if rotated_block.orientation() != 0 {
                            rotated_block.set_orientation(0);
                        }
                        let new_item = Item::Block(rotated_block, amt);
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
                        let new_item = Item::Block(stair_block, amt);
                        self.player.hotbar.set_selected(new_item);
                    }
                    _ => {}
                }
            }
        }
    }

    pub fn rotate_player(&mut self, sensitivity: f32) {
        if self.player.is_dead() || self.display_inventory {
            return;
        }

        let (dmousex, dmousey) = self.get_mouse_diff();
        //Rotate camera
        self.cam.rotate(dmousex, dmousey, sensitivity);
    }

    //Update player and camera
    pub fn update_player(&mut self, dt: f32) {
        if self.player.is_dead() {
            return;
        }

        //Set rotation of player
        self.player.rotation = self.cam.yaw;
        //Update player
        self.player.update(dt, &self.world);
        match self.game_mode() {
            GameMode::Survival => self.player.update_survival(dt, &self.world),
            GameMode::Creative => self.player.update_creative(dt),
        }
        //Set position of camera
        self.cam.position = self.player.position + self.player.cam_offset();

        if self.display_inventory {
            self.player.speed = 0.0;
            return;
        }

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
        //TODO: implement dropped items
        let q = self.get_key_state(Key::Q);
        if q == KeyState::JustPressed && lshift.is_held() {
            //Drop everything in selected slot
            self.player.hotbar.set_selected(Item::Empty);
        } else if q == KeyState::JustPressed {
            self.player.hotbar.drop_selected();
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

        if self.display_debug {
            self.display_inventory = false;
        }
    }

    //Only run in survival mode
    fn handle_block_destruction(&mut self, destroyed: Option<(i32, i32, i32)>, block: Block) {
        if self.game_mode() != GameMode::Survival {
            return;
        }

        if let Some((x, y, z)) = destroyed {
            let held_item = self.player.hotbar.get_selected();
            let block_drop = get_drop(&self.block_info, held_item, block);
            //If it's ice, then set it to be water if there is a non-empty
            //block beneath it, this only applies if nothing is dropped from the ice
            if block.id == 85 && block.shape() == FULL_BLOCK && block_drop.is_empty() {
                break_ice(&mut self.world, x, y, z);
            }
        }
    }

    fn update_tool_durability(&mut self, destroyed_block: Block) {
        let (id, mut tool) = match self.player.hotbar.get_selected() {
            Item::Tool(id, tool) => (id, tool),
            _ => return,
        };

        let block_info = self.block_info.get(&destroyed_block.id);
        let (preferred_tool, break_time) = if let Some(block_info) = block_info {
            (block_info.preferred_tool, block_info.break_time)
        } else {
            (None, 0.0)
        };

        if break_time > 0.0 && Some(tool.tool_type) == preferred_tool {
            tool.update_durability(1);
        } else if break_time > 0.0 && Some(tool.tool_type) != preferred_tool {
            tool.update_durability(2);
        }

        if tool.durability > 0 {
            self.player.hotbar.update_selected(Item::Tool(id, tool));
        } else {
            self.player.hotbar.update_selected(Item::Empty);
        }
    }

    //Returns true if a block has been destroyed
    fn destroy_block(&mut self, chunktables: &mut ChunkTables) -> bool {
        let pos = self.cam.position;
        let dir = self.cam.forward();

        let (x, y, z) = voxel::build::get_selected(pos, dir, &self.world);
        let block = self.world.get_block(x, y, z);

        let stuck = self.player.get_head_stuck_block(&self.world);
        if self.get_mouse_state(MouseButtonLeft).is_held()
            && self.destroy_cooldown <= 0.0
            && stuck.is_none()
        {
            let destroyed = destroy_block(pos, dir, &mut self.world);
            self.handle_block_destruction(destroyed, block);
            let update_mesh = self.world.update_single_block_light(destroyed);
            gfx::update_chunk_vaos(chunktables, destroyed, &self.world);
            for (x, y, z) in update_mesh {
                chunktables.update_table(&self.world, x, y, z);
            }
            if destroyed.is_some() {
                self.destroy_cooldown = BUILD_COOLDOWN;
                return true;
            } else {
                self.destroy_cooldown = 0.0;
                return false;
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
                return true;
            } else {
                self.destroy_cooldown = 0.0;
                return false;
            }
        }
        false
    }

    fn destroy_blocks_survival(&mut self, chunktables: &mut ChunkTables, dt: f32) {
        let pos = self.cam.position;
        let dir = self.cam.forward();
        let new_target = voxel::build::get_selected(pos, dir, &self.world);

        let submerged = self.player.head_intersection(&self.world, 13)
            || self.player.head_intersection(&self.world, 12);

        //Slow down mining if submerged or suffocating
        let mut multiplier = 1.0;

        if submerged || self.player.suffocating(&self.world) {
            multiplier *= 0.2;
        }

        if self.player.is_falling() {
            multiplier *= 0.2;
        }

        if self.player.target_block == Some(new_target)
            && self.get_mouse_state(MouseButtonLeft).is_held()
        {
            self.player.break_timer += dt * multiplier;
        } else {
            self.player.break_timer = 0.0;
        }

        self.player.target_block = Some(new_target);

        let (x, y, z) = new_target;
        let block = self.world.get_block(x, y, z);
        let info = self.get_block_info(block.id);

        let held = self.player.hotbar.get_selected();
        let break_time = info.get_break_time(held);
        if self.player.break_timer > break_time && block.id != EMPTY_BLOCK {
            if self.destroy_block(chunktables) {
                let drop = get_drop(&self.block_info, held, block);
                //TODO: implement dropped items
                //If the player does not have enough space in their inventory,
                //just drop the items on the ground
                self.player.add_item(drop);
                self.update_tool_durability(block);
            }
            self.player.break_timer = 0.0;
        }
    }

    //Returns true if a block was placed, false otherwise
    fn place_block(&mut self, chunktables: &mut ChunkTables) -> bool {
        //Place blocks
        if !self.get_mouse_state(MouseButtonRight).is_held() {
            self.build_cooldown = 0.0;
        }

        let pos = self.cam.position;
        let dir = self.cam.forward();

        let stuck = self.player.get_head_stuck_block(&self.world);
        if self.get_mouse_state(MouseButtonRight).is_held()
            && self.build_cooldown <= 0.0
            && stuck.is_none()
        {
            //Attempt to interact with a block
            let interacted = interact_with_block(pos, dir, &mut self.world, &self.player);
            if interacted.is_some() {
                let update_mesh = self.world.update_single_block_light(interacted);
                gfx::update_chunk_vaos(chunktables, interacted, &self.world);
                for (x, y, z) in update_mesh {
                    chunktables.update_table(&self.world, x, y, z);
                }
                self.hand_animation = 0.1;
                self.build_cooldown = BUILD_COOLDOWN;
                //No block placed
                return false;
            }

            let placed = place_block(pos, dir, &mut self.world, &self.player);
            let update_mesh = self.world.update_single_block_light(placed);
            gfx::update_chunk_vaos(chunktables, placed, &self.world);
            for (x, y, z) in update_mesh {
                chunktables.update_table(&self.world, x, y, z);
            }
            if placed.is_some() {
                self.hand_animation = 0.1;
                self.build_cooldown = BUILD_COOLDOWN;
                return true;
            } else {
                self.build_cooldown = 0.0;
                return false;
            }
        }
        false
    }

    //Returns true if the hoe was used to till dirt
    fn use_hoe(&mut self, chunktables: &mut ChunkTables) -> bool {
        if !self.get_mouse_state(MouseButtonRight).is_held() {
            self.build_cooldown = 0.0;
            return false;
        }

        if self.build_cooldown > 0.0 {
            return false;
        }

        let pos = self.cam.position;
        let dir = self.cam.forward();

        //Attempt to interact with a block
        let interacted = interact_with_block(pos, dir, &mut self.world, &self.player);
        if interacted.is_some() {
            let update_mesh = self.world.update_single_block_light(interacted);
            gfx::update_chunk_vaos(chunktables, interacted, &self.world);
            for (x, y, z) in update_mesh {
                chunktables.update_table(&self.world, x, y, z);
            }
            self.hand_animation = 0.1;
            self.build_cooldown = BUILD_COOLDOWN;
            return false;
        }

        if let Some((x, y, z)) = self.player.target_block {
            let target = self.world.get_block(x, y, z);
            let above = self.world.get_block(x, y + 1, z);
            if !above.transparent() && above.id != EMPTY_BLOCK {
                self.place_block(chunktables);
                return false;
            }
            //Ignore non dirt and non grass blocks
            if target.id != 1 && target.id != 4 && target.id != 87 {
                self.place_block(chunktables);
                return false;
            }
            if target.shape() != FULL_BLOCK {
                self.place_block(chunktables);
                return false;
            }
            self.hand_animation = 0.1;
            //Set it to be dry farmland
            self.world.set_block(x, y, z, Block::new_id(45));
            let update_mesh = self.world.update_single_block_light(Some((x, y, z)));
            gfx::update_chunk_vaos(chunktables, Some((x, y, z)), &self.world);
            for (x, y, z) in update_mesh {
                chunktables.update_table(&self.world, x, y, z);
            }
        }
        true
    }

    //Returns true if the player can eat
    fn can_eat(&mut self, chunktables: &mut ChunkTables) -> bool {
        if !self.get_mouse_state(MouseButtonRight).is_held() {
            self.build_cooldown = 0.0;
            return false;
        }

        let pos = self.cam.position;
        let dir = self.cam.forward();

        //Attempt to interact with a block
        let interacted = if self.build_cooldown <= 0.0 && self.eat_animation <= 0.0 {
            interact_with_block(pos, dir, &mut self.world, &self.player)
        } else {
            None
        };
        if interacted.is_some() {
            let update_mesh = self.world.update_single_block_light(interacted);
            gfx::update_chunk_vaos(chunktables, interacted, &self.world);
            for (x, y, z) in update_mesh {
                chunktables.update_table(&self.world, x, y, z);
            }
            self.hand_animation = 0.1;
            self.build_cooldown = BUILD_COOLDOWN;
            return false;
        }

        if self.player.health == DEFAULT_MAX_HEALTH && self.player.stamina >= 0.99 {
            return false;
        }

        true
    }

    fn use_bucket(&mut self, chunktables: &mut ChunkTables, blockid: u8) { 
        if blockid == 0 {
            if self.get_mouse_state(MouseButtonRight) != KeyState::JustPressed {
                return;
            }
            
            if self.build_cooldown > 0.0 {
                return;
            }
            //Empty bucket, attempt to pick up water or lava
            //Attempt to get the fluid block the player is targeting
            let pos = self.cam.position;
            let dir = self.cam.forward();
            let (x, y, z) = voxel::build::get_selected_fluid(pos, dir, &self.world);
            let block = self.world.get_block(x, y, z);
            if block.is_fluid() && block.geometry == 7 {
                self.world.set_block(x, y, z, Block::new());
                self.player.hotbar.update_selected(Item::Bucket(block.id));
                let update_mesh = self.world.update_single_block_light(Some((x, y, z)));
                gfx::update_chunk_vaos(chunktables, Some((x, y, z)), &self.world);
                for (x, y, z) in update_mesh {
                    chunktables.update_table(&self.world, x, y, z);
                }
                self.hand_animation = 0.1;
                self.build_cooldown = BUILD_COOLDOWN;
            }
        } else {
            if self.get_mouse_state(MouseButtonRight) != KeyState::JustPressed {
                return;
            }

            if self.place_block(chunktables) {
                self.player.hotbar.update_selected(Item::Bucket(0));
            }
        }
    }

    fn use_hand_item(&mut self, chunktables: &mut ChunkTables, dt: f32) {
        let selected = self.player.hotbar.get_selected();
        let selected_str = item_to_string(selected);
        let leftover = self
            .leftover_table
            .get(&selected_str)
            .cloned()
            .unwrap_or(Item::Empty);
        match selected {
            Item::Block(_block, _amt) => {
                let placed = self.place_block(chunktables);
                //Use item in survival mode
                if placed && self.game_mode() == GameMode::Survival {
                    let item = remove_amt_item(selected, 1);
                    self.player.hotbar.update_selected(item);
                }
            }
            Item::Tool(id, info) => {
                if info.tool_type == ToolType::Hoe {
                    if self.use_hoe(chunktables) {
                        let mut info_copy = info;
                        if self.game_mode() == GameMode::Survival {
                            info_copy.update_durability(1);
                        }
                        let updated_tool = if info_copy.durability > 0 {
                            Item::Tool(id, info_copy)
                        } else {
                            Item::Empty
                        };
                        self.player.hotbar.update_selected(updated_tool)
                    }
                } else {
                    self.place_block(chunktables);
                }
            }
            Item::Food(_id, info) => {
                if self.can_eat(chunktables) {
                    self.eat_animation += dt * 1.33;
                } else {
                    self.eat_animation = 0.0;
                }

                if self.eat_animation > 1.0 {
                    self.player.eat(info);
                    self.player.hotbar.update_selected(leftover);
                    self.eat_animation = 0.0;
                }
            }
            Item::Bucket(blockid) => {
                self.use_bucket(chunktables, blockid);
            }
            _ => {
                self.place_block(chunktables);
            }
        }
    }

    //Place and destroy blocks
    pub fn build(&mut self, chunktables: &mut ChunkTables, dt: f32) {
        if self.player.is_dead() || self.display_inventory {
            return;
        }

        //Destroy blocks
        if !self.get_mouse_state(MouseButtonLeft).is_held() {
            self.destroy_cooldown = 0.0;
        }

        match self.game_mode() {
            GameMode::Creative => {
                self.destroy_block(chunktables);
            }
            GameMode::Survival => self.destroy_blocks_survival(chunktables, dt),
        }

        self.use_hand_item(chunktables, dt);
    }

    //Toggle pause screens
    pub fn toggle_pause_screens(&mut self) {
        if self.get_key_state(Key::Escape) == KeyState::JustPressed {
            //Escape out of the block menu
            if self.display_block_menu {
                self.display_block_menu = false;
                self.paused = false;
                return;
            }

            //Escape out of inventory
            if self.display_inventory {
                self.display_inventory = false;
                self.paused = false;
                return;
            }

            //Pause the game
            self.paused = !self.paused;
        }

        if self.get_key_state(Key::E) == KeyState::JustPressed
            && !self.display_debug
            && !self.paused
        {
            self.display_inventory = !self.display_inventory;
            self.display_block_menu = false;
            self.paused = false;
            //Reset player mining progress
            self.player.break_timer = 0.0;
            self.hand_animation = 0.0;
        }

        //Toggle the block menu with Tab (Note: the block menu pauses the game)
        //Only enable block menu in creative mode
        if self.game_mode() == GameMode::Survival {
            self.display_block_menu = false;
            return;
        }

        if self.get_key_state(Key::Tab) == KeyState::JustPressed {
            self.display_inventory = false;
            self.display_block_menu = !self.display_block_menu;
            self.paused = self.display_block_menu;
        }
    }

    //Handle pausing
    pub fn pause(&mut self) {
        let inventory_previously_open = self.display_inventory;
        self.toggle_pause_screens();
        if inventory_previously_open && !self.display_inventory {
            //Close inventory
            self.close_inventory();
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

    pub fn get_eat_animation(&self) -> f32 {
        self.eat_animation
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
        if self.player.is_dead() || self.display_inventory {
            return;
        }

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

    //Respawn player if they are dead
    pub fn respawn(&mut self) {
        if !self.player.is_dead() {
            return;
        }

        //Respawn player
        self.player = self.player.respawn(7.5, 7.5);
        self.player.position.y = 128.0;
        self.world.update_generation_queue(self.player.position);

        //Save world
        self.save_entire_world();

        let path = self.world.path.clone();
        let mut temp_world = World::load_world_metadata(&path);
        let pos = self.player.position;
        temp_world.load_for_respawn(pos.x, pos.y, pos.z);
        //Attempt to set up player y position
        if self.player.check_collision(&temp_world).is_some() {
            //Look upwards for a spawn position
            self.player.position.y = 127.0 + PLAYER_HEIGHT / 2.0;
            while self.player.check_collision(&temp_world).is_some() {
                self.player.position.y += 1.0;
                let pos = self.player.position;
                temp_world.load_for_respawn(pos.x, pos.y, pos.z);
            }
        } else {
            //Look downwards for a spawn position
            for ref y in (-64..=128).rev() {
                self.player.position.y = *y as f32 + PLAYER_HEIGHT / 2.0;
                let pos = self.player.position;
                temp_world.load_for_respawn(pos.x, pos.y, pos.z);
                if self.player.check_collision(&temp_world).is_some() {
                    self.player.position.y += 1.0;
                    break;
                }
            }
        }

        self.world = World::load_world_metadata(&path);
        self.world.update_generation_queue(self.player.position);
        self.world.load_chunks();
        self.world.init_block_light();
        self.world.init_sky_light();

        //Set up camera
        self.cam.position = self.player.position;
        self.cam.pitch = 0.0;
        self.cam.yaw = self.player.rotation;
        self.save_entire_world();

        eprintln!("player respawned.");
    }

    pub fn close_inventory(&mut self) {
        self.prev_selected_slot = "".to_string();

        //Items to drop
        let mut items = vec![];
        //Attempt to add the mouse item to the inventory
        items.push(self.player.mouse_item);
        self.player.mouse_item = Item::Empty;
        //Attempt to add items in the crafting grid to the inventory
        for iy in 0..self.player.crafting_grid.h() {
            for ix in 0..self.player.crafting_grid.w() {
                let item = self.player.crafting_grid.get_item(ix, iy);
                items.push(item);
            }
        }
        self.player.crafting_grid.clear();

        let mut to_drop = vec![];
        for item in items {
            let leftover = self.player.add_item(item);
            if !leftover.is_empty() {
                to_drop.push(leftover);
            }
        }

        //If that is not possible, drop it on the ground
        //TODO: implement dropped items
    }
}
