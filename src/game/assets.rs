pub mod models;
pub mod shaders;
pub mod textures;

use super::{inventory::load_leftover_table, Game};
use egui_backend::egui::{FontData, FontDefinitions, FontFamily};
use egui_gl_glfw as egui_backend;
use std::{fs::File, io::Read};

pub fn load_font(path: &str, fonts: &mut FontDefinitions) {
    let font_file = File::open(path);
    match font_file {
        Ok(mut font_file) => {
            let mut bytes = vec![];
            let res = font_file.read_to_end(&mut bytes);
            match res {
                Ok(sz) => eprintln!("read {sz} bytes from {path}"),
                Err(msg) => eprintln!("{msg}"),
            }
            fonts
                .font_data
                .insert("font".to_string(), FontData::from_owned(bytes));
        }
        Err(msg) => {
            eprintln!("Failed to open: {path}");
            eprintln!("{msg}");
        }
    }

    if let Some(prop) = fonts.families.get_mut(&FontFamily::Proportional) {
        prop.insert(0, "font".to_string());
    }

    if let Some(mono) = fonts.families.get_mut(&FontFamily::Monospace) {
        mono.insert(0, "font".to_string());
    }
}

impl Game {
    pub fn get_font(&self) -> FontDefinitions {
        self.fonts.clone()
    }

    pub fn load_assets(&mut self) {
        load_font(&self.cfg.font_path, &mut self.fonts);
        self.models.add_default_models();
        self.shaders.load_shaders("assets/shaders.impfile");
        self.textures.load_textures("assets/textures.impfile");

        self.load_block_info("assets/block_info.impfile");
        self.recipe_table
            .load_recipes("assets/item_aliases.impfile", "assets/recipes.impfile");
        self.leftover_table =
            load_leftover_table("assets/item_aliases.impfile", "assets/leftovers.impfile");
    }
}
