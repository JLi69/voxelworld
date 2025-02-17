pub mod buildchunk;
pub mod chunktable;
pub mod display;
mod face_data;
pub mod fluid;
pub mod frustum;
pub mod models;
pub mod nonvoxel;

use crate::game::{Camera, Game};
use buildchunk::{generate_chunk_vertex_data, ChunkData};
use cgmath::Matrix4;
pub use chunktable::{update_chunk_vaos, ChunkTables, ChunkVaoTable};
use glfw::PWindow;

pub fn calculate_perspective(window: &PWindow, cam: &Camera) -> Matrix4<f32> {
    let (w, h) = window.get_size();
    let aspect = w as f32 / h as f32;
    cgmath::perspective(cam.get_fovy(), aspect, cam.znear, cam.zfar)
}

//Returns the aspect ratio of the window as an f32
pub fn calculate_aspect(window: &PWindow) -> f32 {
    let (w, h) = window.get_size();
    w as f32 / h as f32
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
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        gl::ClearColor(0.4, 0.8, 1.0, 1.0);
    }
}

pub fn set_nondefault_background_color(gamestate: &Game) {
    if gamestate.player.head_intersection(&gamestate.world, 12) {
        unsafe {
            gl::ClearColor(0.16, 0.41, 0.51, 1.0);
        }
    } else if gamestate.player.head_intersection(&gamestate.world, 13) {
        unsafe {
            gl::ClearColor(1.0, 0.3, 0.0, 1.0);
        }
    }
}
