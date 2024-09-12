pub mod chunk;
mod face_data;

use self::chunk::generate_chunk_vertex_data;
use crate::voxel::{ChunkPos, World};
use cgmath::Matrix4;
use chunk::ChunkData;
use glfw::PWindow;
use std::mem::size_of;
use std::os::raw::c_void;

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
        //TODO: uncomment line below and fix the orientation of block faces
        //gl::Enable(gl::CULL_FACE);
    }
}

pub struct ChunkVaoTable {
    pub vaos: Vec<u32>,
    pub buffers: Vec<u32>,
    pub vertex_count: Vec<i32>,
    pub chunk_positions: Vec<ChunkPos>,
}

impl ChunkVaoTable {
    pub fn new(count: usize) -> Self {
        Self {
            vaos: vec![0; count],
            buffers: vec![0; count],
            vertex_count: vec![0; count],
            chunk_positions: vec![ChunkPos::origin(); count],
        }
    }

    pub fn generate_chunk_vaos(&mut self, world: &World) {
        unsafe {
            gl::GenVertexArrays(self.vaos.len() as i32, &mut self.vaos[0]);
            gl::GenBuffers(self.buffers.len() as i32, &mut self.buffers[0]);
        }

        for i in 0..world.get_chunk_count() {
            let chunk = world.get_chunk_by_idx(i);
            let chunkpos = chunk.get_chunk_pos();
            let adj_chunks = [
                world.get_chunk(chunkpos.x, chunkpos.y + 1, chunkpos.z),
                world.get_chunk(chunkpos.x, chunkpos.y - 1, chunkpos.z),
                world.get_chunk(chunkpos.x - 1, chunkpos.y, chunkpos.z),
                world.get_chunk(chunkpos.x + 1, chunkpos.y, chunkpos.z),
                world.get_chunk(chunkpos.x, chunkpos.y, chunkpos.z - 1),
                world.get_chunk(chunkpos.x, chunkpos.y, chunkpos.z + 1),
            ];

            let chunkdata = generate_chunk_vertex_data(chunk, adj_chunks);
            self.chunk_positions[i] = chunkpos;
            self.vertex_count[i] = chunkdata.len() as i32 / 4;
            send_chunk_data_to_vao(self.vaos[i], self.buffers[i], &chunkdata)
        }
    }
}

fn send_chunk_data_to_vao(vao: u32, block_buffer: u32, chunkdata: &ChunkData) {
    if chunkdata.is_empty() {
        return;
    }

    unsafe {
        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, block_buffer);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (chunkdata.len() * size_of::<u8>()) as isize,
            &chunkdata[0] as *const u8 as *const c_void,
            gl::STATIC_DRAW,
        );
        gl::VertexAttribIPointer(
            0,
            4,
            gl::UNSIGNED_BYTE,
            size_of::<u8>() as i32 * 4,
            std::ptr::null::<u8>() as *const c_void,
        );
        gl::EnableVertexAttribArray(0);
    }
}
