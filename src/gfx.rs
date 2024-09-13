pub mod chunk;
mod face_data;

use self::chunk::generate_chunk_vertex_data;
use crate::voxel::{world_to_chunk_position, wrap_coord, Chunk, ChunkPos, World, CHUNK_SIZE_I32};
use cgmath::Matrix4;
use chunk::ChunkData;
use glfw::PWindow;
use std::collections::HashMap;
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
        gl::Enable(gl::CULL_FACE);
    }
}

pub struct ChunkVaoTable {
    pub vaos: Vec<u32>,
    pub buffers: Vec<u32>,
    pub vertex_count: Vec<i32>,
    pub chunk_positions: Vec<ChunkPos>,
    pos_to_idx: HashMap<(i32, i32, i32), usize>,
}

impl ChunkVaoTable {
    pub fn new(count: usize) -> Self {
        Self {
            vaos: vec![0; count],
            buffers: vec![0; count],
            vertex_count: vec![0; count],
            chunk_positions: vec![ChunkPos::origin(); count],
            pos_to_idx: HashMap::new(),
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
            send_chunk_data_to_vao(self.vaos[i], self.buffers[i], &chunkdata);
            let pos = (chunkpos.x, chunkpos.y, chunkpos.z);
            self.pos_to_idx.insert(pos, i);
        }
    }

    fn convert_pos_to_idx(&self, chunkpos: &ChunkPos) -> usize {
        let pos = (chunkpos.x, chunkpos.y, chunkpos.z);
        match self.pos_to_idx.get(&pos) {
            Some(idx) => *idx,
            //Return an out of bounds index if we cannot locate a position
            _ => self.vaos.len(),
        }
    }

    fn update_chunk(&mut self, chunk: Option<&Chunk>, world: &World) {
        if let Some(chunk) = chunk {
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
            let idx = self.convert_pos_to_idx(&chunkpos);
            self.chunk_positions[idx] = chunkpos;
            self.vertex_count[idx] = chunkdata.len() as i32 / 4;
            send_chunk_data_to_vao(self.vaos[idx], self.buffers[idx], &chunkdata);
        }
    }

    fn update_adjacent(
        &mut self,
        adj_chunks: &[Option<&Chunk>; 6],
        x: i32,
        y: i32,
        z: i32,
        world: &World,
    ) {
        let x = wrap_coord(x % CHUNK_SIZE_I32);
        let y = wrap_coord(y % CHUNK_SIZE_I32);
        let z = wrap_coord(z % CHUNK_SIZE_I32);

        if x == CHUNK_SIZE_I32 - 1 {
            self.update_chunk(adj_chunks[3], world);
        } else if x == 0 {
            self.update_chunk(adj_chunks[2], world);
        }

        if y == CHUNK_SIZE_I32 - 1 {
            self.update_chunk(adj_chunks[0], world);
        } else if y == 0 {
            self.update_chunk(adj_chunks[1], world);
        }

        if z == CHUNK_SIZE_I32 - 1 {
            self.update_chunk(adj_chunks[5], world);
        } else if z == 0 {
            self.update_chunk(adj_chunks[4], world);
        }
    }

    pub fn update_chunk_with_adj(&mut self, x: i32, y: i32, z: i32, world: &World) {
        let (chunkx, chunky, chunkz) = world_to_chunk_position(x, y, z);
        let pos = (chunkx, chunky, chunkz);
        if let Some(i) = self.pos_to_idx.get(&pos) {
            let idx = *i;
            let chunk = world.get_chunk_by_idx(idx);
            let chunkpos = chunk.get_chunk_pos();
            let adj_chunks = [
                world.get_chunk(chunkpos.x, chunkpos.y + 1, chunkpos.z),
                world.get_chunk(chunkpos.x, chunkpos.y - 1, chunkpos.z),
                world.get_chunk(chunkpos.x - 1, chunkpos.y, chunkpos.z),
                world.get_chunk(chunkpos.x + 1, chunkpos.y, chunkpos.z),
                world.get_chunk(chunkpos.x, chunkpos.y, chunkpos.z - 1),
                world.get_chunk(chunkpos.x, chunkpos.y, chunkpos.z + 1),
            ];

            self.update_chunk(world.get_chunk(chunkx, chunky, chunkz), world);
            self.update_adjacent(&adj_chunks, x, y, z, world);
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
