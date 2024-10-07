use crate::assets::Texture;
use std::collections::HashMap;

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
    pub fn load_textures(&mut self) {
        let blocktexture = load_texture("assets/textures/blocktextures.png");
        self.textures.insert("blocks".to_string(), blocktexture);
    }
}
