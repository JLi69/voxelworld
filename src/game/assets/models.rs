use crate::gfx;
use crate::gfx::models::Vao;
use std::collections::HashMap;

pub struct ModelManager {
    vaos: HashMap<String, Vao>,
}

impl ModelManager {
    pub fn new() -> Self {
        Self {
            vaos: HashMap::new(),
        }
    }

    //Binds and returns an option with a vao inside
    //returns None if no vao is found at `id`
    pub fn bind(&self, id: &str) -> Option<Vao> {
        if let Some(vao) = self.vaos.get(id) {
            vao.bind();
        } else {
            eprintln!("E: model \"{id}\" not found!")
        }

        self.vaos.get(id).cloned()
    }

    //Adds "default models" should be called at the beginning of the game
    pub fn add_default_models(&mut self) {
        //Generate cube model
        let cube = gfx::models::gen_cube_vao();
        self.vaos.insert("cube".to_string(), cube);
        //Generate 2d quad model
        let quad2d = gfx::models::gen_quad2d_vao();
        self.vaos.insert("quad2d".to_string(), quad2d);
    }
}

//Draws elements for an optional vao, will do nothing if vao is None
pub fn draw_elements(vao: Option<Vao>) {
    if let Some(vao) = vao {
        vao.draw_elements();
    }
}
