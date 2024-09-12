extern crate gl;
use gl::types::*;

use cgmath::{Matrix, Matrix4};
use std::ffi::CString;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::str;

fn read_shader_src_file(path: &str) -> String {
    //Attempt to open the shader source file
    let file = File::open(Path::new(path));

    match file {
        Ok(mut shader_file) => {
            //Attempt to read source code from file
            let mut src = String::new();
            let res = shader_file.read_to_string(&mut src);

            match res {
                Ok(sz) => {
                    eprintln!("read {sz} bytes from {path}");
                }
                Err(msg) => {
                    eprintln!("ERROR: {msg}");
                }
            }

            src
        }
        //Failed to open file
        Err(msg) => {
            eprintln!("ERROR: failed to open: {path}");
            eprintln!("{msg}");
            String::new() //If something went wrong, just return an empty string
        }
    }
}

pub fn create_and_compile_shader(path: &str, shader_type: GLenum) -> u32 {
    let shader = unsafe { gl::CreateShader(shader_type) };

    let src = read_shader_src_file(path);
    let src_cstring_res = CString::new(src.into_bytes());
    unsafe {
        match src_cstring_res {
            Ok(s) => {
                gl::ShaderSource(shader, 1, &s.as_ptr(), std::ptr::null());
                gl::CompileShader(shader);

                //Check for any compilation errors
                let mut status = 0;
                gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status as *mut i32);
                if status != gl::TRUE as i32 {
                    eprintln!("ERROR: Failed to compile: {path}");
                    let mut length = 0;
                    gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut length);
                    let mut log = vec![0; length as usize];
                    gl::GetShaderInfoLog(
                        shader,
                        log.len() as i32,
                        &mut length,
                        log.as_mut_ptr() as *mut GLchar,
                    );

                    //Attempt to output any compile errors
                    match str::from_utf8(&log) {
                        Ok(compile_err) => {
                            eprintln!("{compile_err}");
                            panic!("Failed to compile shader!");
                        }
                        Err(msg) => {
                            eprintln!("{msg}");
                        }
                    }
                }
            }
            Err(msg) => {
                eprintln!("{msg}");
            }
        }
    };

    shader
}

pub struct ShaderProgram {
    program_id: u32,
}

#[allow(dead_code)]
impl ShaderProgram {
    //Creates an empty shader program
    pub fn create_program() -> Self {
        unsafe {
            Self {
                program_id: gl::CreateProgram(),
            }
        }
    }

    //Add and link shaders together to create a shader program
    pub fn add_shaders(&self, shader_ids: &[u32]) {
        unsafe {
            for shader in shader_ids {
                gl::AttachShader(self.program_id, *shader);
            }

            gl::LinkProgram(self.program_id);
            //Output any linking errors
            let mut link_status = 0;
            gl::GetProgramiv(self.program_id, gl::LINK_STATUS, &mut link_status);
            if link_status != gl::TRUE as i32 {
                let mut length = 0;
                gl::GetProgramiv(self.program_id, gl::INFO_LOG_LENGTH, &mut length);
                let mut log = vec![0; length as usize];
                gl::GetProgramInfoLog(
                    self.program_id,
                    log.len() as i32,
                    &mut length,
                    log.as_mut_ptr() as *mut GLchar,
                );

                //Attempt to output any compile errors
                match str::from_utf8(&log) {
                    Ok(compile_err) => {
                        eprintln!("{compile_err}");
                    }
                    Err(msg) => {
                        eprintln!("{msg}");
                    }
                }
            }

            gl::ValidateProgram(self.program_id);

            for shader in shader_ids {
                gl::DetachShader(self.program_id, *shader);
            }
        }
    }

    pub fn use_program(&self) {
        unsafe {
            gl::UseProgram(self.program_id);
        }
    }

    //Returns the location of a uniform
    //If the uniform can't be found (or uniform_name is an invalid c string)
    //then None is returned, otherwise Some(location) is given
    pub fn get_uniform_location(&self, uniform_name: &str) -> Option<i32> {
        let name_cstr_raw = CString::new(uniform_name);
        unsafe {
            match name_cstr_raw {
                Ok(s) => {
                    let location = gl::GetUniformLocation(self.program_id, s.as_ptr());

                    if location < 0 {
                        return None;
                    }

                    Some(location)
                }
                Err(msg) => {
                    eprintln!("{msg}");
                    None
                }
            }
        }
    }

    //Sends a uniform to a location based
    fn uniform<F: Fn(i32)>(&self, uniform_name: &str, uniform_function: F) {
        match self.get_uniform_location(uniform_name) {
            Some(location) => {
                uniform_function(location);
            }
            None => {
                eprintln!("Uniform not found: {uniform_name}")
            }
        }
    }

    //Send a 4 dimensional vector to the shader
    pub fn uniform_vec4f(&self, uniform_name: &str, x: f32, y: f32, z: f32, w: f32) {
        self.uniform(uniform_name, |location| unsafe {
            gl::Uniform4f(location, x, y, z, w);
        });
    }

    //Send a 2 dimensional vector to the shader
    pub fn uniform_vec2f(&self, uniform_name: &str, x: f32, y: f32) {
        self.uniform(uniform_name, |location| unsafe {
            gl::Uniform2f(location, x, y);
        })
    }

    //Send a 3 dimensional vector to the shader
    pub fn uniform_vec3f(&self, uniform_name: &str, x: f32, y: f32, z: f32) {
        self.uniform(uniform_name, |location| unsafe {
            gl::Uniform3f(location, x, y, z);
        })
    }

    //Send a 4 x 4 matrix to the shader
    pub fn uniform_matrix4f(&self, uniform_name: &str, mat: &Matrix4<f32>) {
        self.uniform(uniform_name, |location| unsafe {
            gl::UniformMatrix4fv(location, 1, gl::FALSE, mat.as_ptr());
        })
    }

    //Send a floating point value to the shader
    pub fn uniform_float(&self, uniform_name: &str, v: f32) {
        self.uniform(uniform_name, |location| unsafe {
            gl::Uniform1f(location, v);
        })
    }

    //Send a boolean to the shader
    pub fn uniform_bool(&self, uniform_name: &str, b: bool) {
        self.uniform(uniform_name, |location| unsafe {
            if b {
                gl::Uniform1i(location, gl::TRUE as i32);
            } else if !b {
                gl::Uniform1i(location, gl::FALSE as i32);
            }
        })
    }
}

//Creates a shader from a vertex shader and fragment shader given by a path
pub fn program_from_vert_and_frag(vert_path: &str, frag_path: &str) -> ShaderProgram {
    let shaders = [
        create_and_compile_shader(vert_path, gl::VERTEX_SHADER),
        create_and_compile_shader(frag_path, gl::FRAGMENT_SHADER),
    ];
    let program = ShaderProgram::create_program();
    program.add_shaders(&shaders);
    program
}
