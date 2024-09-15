pub mod buildchunk;
pub mod chunktable;
mod face_data;

use buildchunk::{generate_chunk_vertex_data, ChunkData};
use cgmath::Matrix4;
pub use chunktable::{update_chunk_vaos, ChunkVaoTable};
use glfw::PWindow;

pub fn calculate_perspective(window: &PWindow) -> Matrix4<f32> {
    let (w, h) = window.get_size();
    let aspect = w as f32 / h as f32;
    cgmath::perspective(cgmath::Deg(75.0), aspect, 0.1, 1000.0)
}

pub fn output_errors() {
    unsafe {
        let mut err = gl::GetError();
        while err != gl::NO_ERROR {
            eprintln!("OpenGL Error: {err}");
            err = gl::GetError();
        }
    }
}

pub fn clear() {
    unsafe {
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }
}

pub fn handle_window_resize(w: i32, h: i32) {
    unsafe {
        gl::Viewport(0, 0, w, h);
    }
}

pub fn set_default_gl_state() {
    //Set OpenGL state
    unsafe {
        gl::Enable(gl::DEPTH_TEST);
        gl::DepthFunc(gl::LEQUAL);
        gl::Enable(gl::CULL_FACE);
    }
}
