use crate::{
    assets::Texture,
    impfile::{self, Entry},
};
use std::collections::HashMap;

struct TextureMetaData {
    name: String,
    path: String,
}

impl TextureMetaData {
    pub fn from_entry(entry: &Entry) -> Self {
        Self {
            name: entry.get_name(),
            path: entry.get_var("path"),
        }
    }
}

pub struct TextureManager {
    textures: HashMap<String, Texture>,
}

//Loads a texture, returns an empty texture if it fails
fn load_texture(path: &str) -> Texture {
    match Texture::load_from_file(path) {
        Ok(tex) => tex,
        Err(msg) => {
            eprintln!("Failed to open texture: {path}");
            eprintln!("{msg}");
            Texture::new()
        }
    }
}

impl TextureManager {
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
        }
    }

    //Binds a texture
    pub fn bind(&self, id: &str) {
        if let Some(texture) = self.textures.get(id) {
            texture.bind();
        } else {
            eprintln!("E: texture \"{id}\" not found!\n");
        }
    }

    //Loads all textures, should be called at the beginning of the game
    pub fn load_textures(&mut self, path: &str) {
        let textures = impfile::parse_file(path);
        for entry in textures {
            let metadata = TextureMetaData::from_entry(&entry);
            let texture = load_texture(&metadata.path);
            self.textures.insert(metadata.name, texture);
        }
    }
}
