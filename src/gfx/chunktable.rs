use super::{generate_chunk_vertex_data, ChunkData};
use crate::assets::shader::ShaderProgram;
use crate::game::Game;
use crate::voxel;
use crate::voxel::BLOCK_REACH;
use crate::voxel::{
    world_to_chunk_position, wrap_coord, Chunk, ChunkPos, World, CHUNK_SIZE_I32, EMPTY_BLOCK,
};
use crate::CHUNK_SIZE_F32;
use std::collections::HashMap;
use std::mem::size_of;
use std::os::raw::c_void;

//Send chunk vertex data to a buffer and vao
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

pub struct ChunkVaoTable {
    pub vaos: Vec<u32>,
    pub buffers: Vec<u32>,
    pub vertex_count: Vec<i32>,
    pub chunk_positions: Vec<ChunkPos>,
    pos_to_idx: HashMap<(i32, i32, i32), usize>,
}

impl ChunkVaoTable {
    //Create a new chunk vao table
    pub fn new(count: usize) -> Self {
        Self {
            vaos: vec![0; count],
            buffers: vec![0; count],
            vertex_count: vec![0; count],
            chunk_positions: vec![ChunkPos::origin(); count],
            pos_to_idx: HashMap::new(),
        }
    }

    //Call this to initialize all of the chunk vaos and buffers
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

    //Return the index of a chunk based on a position, returns none if no
    //such position can be found
    fn convert_pos_to_idx(&self, chunkpos: &ChunkPos) -> usize {
        let pos = (chunkpos.x, chunkpos.y, chunkpos.z);
        match self.pos_to_idx.get(&pos) {
            Some(idx) => *idx,
            //Return an out of bounds index if we cannot locate a position
            _ => self.vaos.len(),
        }
    }

    //Update chunk buffer data
    fn update_chunk_vao(&mut self, chunk: Option<&Chunk>, world: &World) {
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

    //Update any adjacent chunks that might also be affected by a block update
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
            self.update_chunk_vao(adj_chunks[3], world);
        } else if x == 0 {
            self.update_chunk_vao(adj_chunks[2], world);
        }

        if y == CHUNK_SIZE_I32 - 1 {
            self.update_chunk_vao(adj_chunks[0], world);
        } else if y == 0 {
            self.update_chunk_vao(adj_chunks[1], world);
        }

        if z == CHUNK_SIZE_I32 - 1 {
            self.update_chunk_vao(adj_chunks[5], world);
        } else if z == 0 {
            self.update_chunk_vao(adj_chunks[4], world);
        }
    }

    //Updates a single chunk and if necessary, also updates the adjacent chunks
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

            self.update_chunk_vao(world.get_chunk(chunkx, chunky, chunkz), world);
            self.update_adjacent(&adj_chunks, x, y, z, world);
        }
    }

    //Displays all the chunk vaos
    pub fn display_chunks(&self, chunkshader: &ShaderProgram, gamestate: &Game) {
        let pos = gamestate.cam.position();
        let dir = gamestate.cam.forward();
        let selected = voxel::raycast(pos, dir, BLOCK_REACH, &gamestate.world);

        let view = gamestate.cam.get_view();
        let (x, y, z) = selected;
        let (x, y, z) = (x.floor(), y.floor(), z.floor());
        let selectedid = gamestate.world.get_block(x as i32, y as i32, z as i32).id;
        chunkshader.uniform_vec3f("selected", x, y, z);
        chunkshader.uniform_bool("selectedEmpty", selectedid == EMPTY_BLOCK);
        chunkshader.uniform_matrix4f("view", &view);
        chunkshader.uniform_matrix4f("persp", &gamestate.persp);

        for (i, vao) in self.vaos.iter().enumerate() {
            if self.vertex_count[i] == 0 {
                continue;
            }

            let pos = self.chunk_positions[i];
            let x = pos.x as f32 * CHUNK_SIZE_F32;
            let y = pos.y as f32 * CHUNK_SIZE_F32;
            let z = pos.z as f32 * CHUNK_SIZE_F32;
            chunkshader.uniform_vec3f("chunkpos", x, y, z);
            unsafe {
                gl::BindVertexArray(*vao);
                gl::DrawArrays(gl::TRIANGLES, 0, self.vertex_count[i]);
            }
        }
    }
}

//Update a chunk table based on a option of a potential block that was changed
pub fn update_chunk_vaos(chunks: &mut ChunkVaoTable, pos: Option<(i32, i32, i32)>, world: &World) {
    if let Some((x, y, z)) = pos {
        chunks.update_chunk_with_adj(x, y, z, world);
    }
}
