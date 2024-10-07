use crate::assets;
use crate::assets::shader::ShaderProgram;
use std::collections::HashMap;

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
    pub fn load_shaders(&mut self) {
        //Create shaders
        //Chunk shader
        let vert = "assets/shaders/chunkvert.glsl";
        let frag = "assets/shaders/chunkfrag.glsl";
        let chunkshader = assets::program_from_vert_and_frag(vert, frag);
        self.shaders.insert("chunk".to_string(), chunkshader);
        //Cube outline shader
        let vert = "assets/shaders/vert.glsl";
        let frag = "assets/shaders/outlinefrag.glsl";
        let outlineshader = assets::program_from_vert_and_frag(vert, frag);
        self.shaders.insert("outline".to_string(), outlineshader);
    }
}
