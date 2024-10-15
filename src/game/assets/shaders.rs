use crate::assets::shader::ShaderProgram;
use crate::impfile::Entry;
use crate::{assets, impfile};
use std::collections::HashMap;

struct ShaderMetaData {
    name: String,
    vert: String,
    frag: String,
}

impl ShaderMetaData {
    pub fn from_entry(entry: &Entry) -> Self {
        Self {
            name: entry.get_name(),
            vert: entry.get_var("vert"),
            frag: entry.get_var("frag"),
        }
    }
}

pub struct ShaderManager {
    shaders: HashMap<String, ShaderProgram>,
}

impl ShaderManager {
    pub fn new() -> Self {
        Self {
            shaders: HashMap::new(),
        }
    }

    //Gets a shader without activating it
    pub fn get(&self, id: &str) -> ShaderProgram {
        self.shaders
            .get(id)
            .copied()
            .unwrap_or(ShaderProgram::zero())
    }

    //Calls use_program() on a shader, if no shader is found at that id None is
    //returned, otherwise Some(shader) is returned
    pub fn use_program(&self, id: &str) -> ShaderProgram {
        if let Some(shader) = self.shaders.get(id) {
            shader.use_program();
        } else {
            eprintln!("E: shader \"{id}\" not found!")
        }

        self.get(id)
    }

    //Loads shaders, should be called at the beginning of the program
    pub fn load_shaders(&mut self, path: &str) {
        let shaders = impfile::parse_file(path);
        for entry in shaders {
            let metadata = ShaderMetaData::from_entry(&entry);
            let shader = assets::program_from_vert_and_frag(&metadata.vert, &metadata.frag);
            self.shaders.insert(metadata.name, shader);
        }
    }
}
