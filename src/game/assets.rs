use super::Game;
use egui_backend::egui::{FontData, FontDefinitions, FontFamily};
use egui_gl_glfw as egui_backend;
use std::{fs::File, io::Read};

impl Game {
    pub fn load_font(&mut self) {
        //Load fonts
        //This font path should probably be a value to be configured
        let path = "assets/fonts/pixeloid/PixeloidSans.ttf".to_string();
        let font_file = File::open(&path);
        match font_file {
            Ok(mut font_file) => {
                let mut bytes = vec![];
                let res = font_file.read_to_end(&mut bytes);
                match res {
                    Ok(sz) => eprintln!("read {sz} bytes from {path}"),
                    Err(msg) => eprintln!("{msg}"),
                }
                self.fonts
                    .font_data
                    .insert("font".to_string(), FontData::from_owned(bytes));
            }
            Err(msg) => {
                eprintln!("Failed to open: {path}");
                eprintln!("{msg}");
            }
        }

        if let Some(prop) = self.fonts.families.get_mut(&FontFamily::Proportional) {
            prop.insert(0, "font".to_string());
        }

        if let Some(mono) = self.fonts.families.get_mut(&FontFamily::Monospace) {
            mono.insert(0, "font".to_string());
        }
    }

    pub fn get_font(&self) -> FontDefinitions {
        self.fonts.clone()
    }

    pub fn load_assets(&mut self) {
        self.load_font();
    }
}
