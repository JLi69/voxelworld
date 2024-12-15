use super::fluid::generate_fluid_vertex_data;
use super::frustum::Frustum;
use super::{generate_chunk_vertex_data, ChunkData};
use crate::assets::shader::ShaderProgram;
use crate::game::physics::Hitbox;
use crate::game::Game;
use crate::voxel::{world_to_chunk_position, wrap_coord, Chunk, ChunkPos, World, CHUNK_SIZE_I32};
use crate::CHUNK_SIZE_F32;
use cgmath::Vector3;
use std::collections::{HashMap, VecDeque};
use std::mem::size_of;
use std::os::raw::c_void;

//Set fog color
fn set_fog(gamestate: &Game, chunkshader: &ShaderProgram) {
    if gamestate.player.head_intersection(&gamestate.world, 12) {
        //Water
        chunkshader.uniform_float("fogdist", -CHUNK_SIZE_F32 / 3.0);
        chunkshader.uniform_float("fogstrength", 1.0 / CHUNK_SIZE_F32);
        chunkshader.uniform_vec4f("fogcolor", 0.16, 0.41, 0.51, 1.0);
    } else if gamestate.player.head_intersection(&gamestate.world, 13) {
        //Lava
        chunkshader.uniform_float("fogdist", -1.8 / 8.0);
        chunkshader.uniform_float("fogstrength", 1.0 / 1.8);
        chunkshader.uniform_vec4f("fogcolor", 1.0, 0.3, 0.0, 1.0);
    } else {
        //Normal
        let range = gamestate.world.get_range() as f32 * CHUNK_SIZE_F32;
        let dist = range * 0.7;
        chunkshader.uniform_float("fogdist", dist);
        chunkshader.uniform_float("fogstrength", 1.0 / (range * 0.2));
        chunkshader.uniform_vec4f("fogcolor", 0.4, 0.8, 1.0, 1.0);
    }
}

const BUF_COUNT: usize = 2;

struct ChunkVao {
    id: u32,
    buffers: [u32; BUF_COUNT],
    vert_count: i32,
}

//Send chunk vertex data to a buffer and vao
fn send_chunk_data_to_vao(chunkvao: &ChunkVao, chunkdata: &ChunkData) {
    if chunkdata.is_empty() {
        return;
    }

    unsafe {
        gl::BindVertexArray(chunkvao.id);
        gl::BindBuffer(gl::ARRAY_BUFFER, chunkvao.buffers[0]);
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
            size_of::<u8>() as i32 * 5,
            std::ptr::null::<u8>() as *const c_void,
        );
        gl::EnableVertexAttribArray(0);
        gl::BindBuffer(gl::ARRAY_BUFFER, chunkvao.buffers[1]);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (chunkdata.len() * size_of::<u8>()) as isize,
            &chunkdata[0] as *const u8 as *const c_void,
            gl::STATIC_DRAW,
        );
        gl::VertexAttribIPointer(
            1,
            1,
            gl::UNSIGNED_BYTE,
            size_of::<u8>() as i32 * 5,
            (size_of::<u8>() * 4) as *const c_void,
        );
        gl::EnableVertexAttribArray(1);
    }
}

pub struct ChunkVaoTable {
    vaos: HashMap<(i32, i32, i32), ChunkVao>,
    to_update: VecDeque<(i32, i32, i32)>,
}

impl ChunkVaoTable {
    //Create a new chunk vao table
    pub fn new() -> Self {
        Self {
            vaos: HashMap::new(),
            to_update: VecDeque::new(),
        }
    }

    pub fn add_to_update(&mut self, x: i32, y: i32, z: i32) {
        self.to_update.push_back((x, y, z));
    }

    fn update_chunk(
        &mut self,
        world: &World,
        gen_verts: fn(&Chunk, &World) -> ChunkData,
    ) {
        let top = self.to_update.pop_front();
        if let Some(pos) = top {
            let (x, y, z) = pos;
            if self.vaos.contains_key(&pos) {
                self.update_chunk_vao(world.get_chunk(x, y, z), world, gen_verts);
            } else {
                //Create a new vao to be added
                let mut vao = ChunkVao {
                    id: 0,
                    buffers: [0; BUF_COUNT],
                    vert_count: 0,
                };

                unsafe {
                    gl::GenVertexArrays(1, &mut vao.id);
                    gl::GenBuffers(BUF_COUNT as i32, &mut vao.buffers[0]);
                }

                if let Some(chunk) = world.get_chunk(x, y, z) {
                    let chunkpos = chunk.get_chunk_pos();
                    let chunkdata = gen_verts(chunk, world);
                    send_chunk_data_to_vao(&vao, &chunkdata);
                    vao.vert_count = chunkdata.len() as i32 / 5;
                    self.vaos.insert((chunkpos.x, chunkpos.y, chunkpos.z), vao);
                }
            }
        }
    }

    pub fn update_chunks(
        &mut self,
        world: &World,
        gen_verts: fn(&Chunk, &World) -> ChunkData,
    ) {
        let start = std::time::Instant::now();
        let mut total_time = 0.0;
        while total_time < 0.002 && !self.to_update.is_empty() {
            self.update_chunk(world, gen_verts);
            let now = std::time::Instant::now();
            total_time = (now - start).as_secs_f32();
        }
    }

    pub fn delete_chunks(&mut self, centerx: i32, centery: i32, centerz: i32, range: i32) {
        let mut vaos_to_delete = vec![];
        let mut buf_to_delete = vec![];
        let mut to_delete = vec![]; //Positions to delete from the map
        for ((x, y, z), vao) in &mut self.vaos {
            if (centerx - x).abs() <= range
                && (centery - y).abs() <= range
                && (centerz - z).abs() <= range
            {
                continue;
            }

            vaos_to_delete.push(vao.id);
            for buf in vao.buffers {
                buf_to_delete.push(buf);
            }

            to_delete.push((*x, *y, *z));
        }

        for pos in to_delete {
            self.vaos.remove(&pos);
        }

        unsafe {
            if !buf_to_delete.is_empty() {
                gl::DeleteBuffers(buf_to_delete.len() as i32, &buf_to_delete[0]);
            }

            if !vaos_to_delete.is_empty() {
                gl::DeleteVertexArrays(vaos_to_delete.len() as i32, &vaos_to_delete[0]);
            }
        }
    }

    //Call this to initialize all of the chunk vaos and buffers
    pub fn generate_chunk_vaos(
        &mut self,
        world: &World,
        gen_verts: fn(&Chunk, &World) -> ChunkData,
    ) {
        let mut vaos = vec![0; world.chunks.len()];
        let mut buffers = vec![0; world.chunks.len() * BUF_COUNT];

        unsafe {
            gl::GenVertexArrays(vaos.len() as i32, &mut vaos[0]);
            gl::GenBuffers(buffers.len() as i32, &mut buffers[0]);
        }

        for (i, chunk) in world.chunks.values().enumerate() {
            let chunkpos = chunk.get_chunk_pos();
            let chunkdata = gen_verts(chunk, world);
            let chunkvao = ChunkVao {
                id: vaos[i],
                buffers: [buffers[i * BUF_COUNT], buffers[i * BUF_COUNT + 1]],
                vert_count: chunkdata.len() as i32 / 5,
            };
            send_chunk_data_to_vao(&chunkvao, &chunkdata);
            self.vaos
                .insert((chunkpos.x, chunkpos.y, chunkpos.z), chunkvao);
        }
    }

    //Update chunk buffer data
    fn update_chunk_vao(
        &mut self,
        chunk: Option<&Chunk>,
        world: &World,
        gen_verts: fn(&Chunk, &World) -> ChunkData,
    ) {
        if let Some(chunk) = chunk {
            let chunkpos = chunk.get_chunk_pos();
            let chunkdata = gen_verts(chunk, world);
            let chunk = self.vaos.get_mut(&(chunkpos.x, chunkpos.y, chunkpos.z));
            if let Some(chunk) = chunk {
                chunk.vert_count = chunkdata.len() as i32 / 5;
                send_chunk_data_to_vao(chunk, &chunkdata);
            }
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
        gen_verts: fn(&Chunk, &World) -> ChunkData,
    ) {
        let x = wrap_coord(x % CHUNK_SIZE_I32);
        let y = wrap_coord(y % CHUNK_SIZE_I32);
        let z = wrap_coord(z % CHUNK_SIZE_I32);

        if x == CHUNK_SIZE_I32 - 1 {
            self.update_chunk_vao(adj_chunks[3], world, gen_verts);
        } else if x == 0 {
            self.update_chunk_vao(adj_chunks[2], world, gen_verts);
        }

        if y == CHUNK_SIZE_I32 - 1 {
            self.update_chunk_vao(adj_chunks[0], world, gen_verts);
        } else if y == 0 {
            self.update_chunk_vao(adj_chunks[1], world, gen_verts);
        }

        if z == CHUNK_SIZE_I32 - 1 {
            self.update_chunk_vao(adj_chunks[5], world, gen_verts);
        } else if z == 0 {
            self.update_chunk_vao(adj_chunks[4], world, gen_verts);
        }
    }

    //Updates a single chunk and if necessary, also updates the adjacent chunks
    pub fn update_chunk_with_adj(
        &mut self,
        x: i32,
        y: i32,
        z: i32,
        world: &World,
        gen_verts: fn(&Chunk, &World) -> ChunkData,
    ) {
        let (chunkx, chunky, chunkz) = world_to_chunk_position(x, y, z);
        let chunk = world.get_chunk(chunkx, chunky, chunkz);
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

            self.update_chunk_vao(world.get_chunk(chunkx, chunky, chunkz), world, gen_verts);
            self.update_adjacent(&adj_chunks, x, y, z, world, gen_verts);
        }
    }

    //Displays all the chunk vaos
    pub fn display_chunks(&self, gamestate: &Game, shaderid: &str) -> u32 {
        gamestate.textures.bind("blocks");
        //Calculate view frustum
        let view_frustum = Frustum::new(&gamestate.cam, gamestate.aspect);

        let chunkshader = gamestate.shaders.use_program(shaderid);
        let view = gamestate.cam.get_view();
        chunkshader.uniform_matrix4f("view", &view);
        chunkshader.uniform_matrix4f("persp", &gamestate.persp);
        chunkshader.uniform_vec3f(
            "campos",
            gamestate.cam.position.x,
            gamestate.cam.position.y,
            gamestate.cam.position.z,
        );

        //Set fog color
        set_fog(gamestate, &chunkshader);

        let mut drawn_count = 0;
        for ((chunkx, chunky, chunkz), vao) in &self.vaos {
            if vao.vert_count == 0 {
                continue;
            }

            let pos = ChunkPos::new(*chunkx, *chunky, *chunkz);
            let x = pos.x as f32 * CHUNK_SIZE_F32;
            let y = pos.y as f32 * CHUNK_SIZE_F32;
            let z = pos.z as f32 * CHUNK_SIZE_F32;

            //Calculate Chunk AABB
            let sz = CHUNK_SIZE_F32;
            let chunkcenter = Vector3::new(x + sz / 2.0, y + sz / 2.0, z + sz / 2.0);
            let aabb = Hitbox::from_vecs(chunkcenter, Vector3::new(sz, sz, sz));
            if !view_frustum.intersects(&aabb) {
                continue;
            }

            drawn_count += 1;
            chunkshader.uniform_vec3f("chunkpos", x, y, z);
            unsafe {
                gl::BindVertexArray(vao.id);
                gl::DrawArrays(gl::TRIANGLES, 0, vao.vert_count);
            }
        }

        drawn_count
    }

    pub fn display_with_backface(&self, gamestate: &Game, shaderid: &str) -> u32 {
        unsafe {
            gl::Disable(gl::CULL_FACE);
        }
        let count = self.display_chunks(gamestate, shaderid);
        unsafe {
            gl::Enable(gl::CULL_FACE);
        }

        count
    }

    //Delete all buffers and vaos
    pub fn clear(&mut self) {
        if self.vaos.is_empty() {
            return;
        }

        let mut buffers = vec![];
        let mut vaoids = vec![];

        for vao in self.vaos.values() {
            vaoids.push(vao.id);
            for buf in vao.buffers {
                buffers.push(buf)
            }
        }

        unsafe {
            gl::DeleteBuffers(buffers.len() as i32, &buffers[0]);
            gl::DeleteVertexArrays(vaoids.len() as i32, &vaoids[0]);
        }

        self.to_update.clear();
        self.vaos.clear();
    }
}

impl Drop for ChunkVaoTable {
    fn drop(&mut self) {
        self.clear();
    }
}

//Update a chunk table based on a option of a potential block that was changed
pub fn update_chunk_vaos(chunks: &mut ChunkTables, pos: Option<(i32, i32, i32)>, world: &World) {
    if let Some((x, y, z)) = pos {
        chunks
            .chunk_vaos
            .update_chunk_with_adj(x, y, z, world, |chunk, world| {
                generate_chunk_vertex_data(chunk, world.get_adjacent(chunk))
            });
        chunks
            .lava_vaos
            .update_chunk_with_adj(x, y, z, world, |chunk, world| {
                generate_fluid_vertex_data(chunk, world.get_adjacent(chunk), world, 13)
            });
        chunks
            .water_vaos
            .update_chunk_with_adj(x, y, z, world, |chunk, world| {
                generate_fluid_vertex_data(chunk, world.get_adjacent(chunk), world, 12)
            });
    }
}

pub struct ChunkTables {
    pub chunk_vaos: ChunkVaoTable,
    pub lava_vaos: ChunkVaoTable,
    pub water_vaos: ChunkVaoTable,
}

impl ChunkTables {
    pub fn new() -> Self {
        Self {
            chunk_vaos: ChunkVaoTable::new(),
            lava_vaos: ChunkVaoTable::new(),
            water_vaos: ChunkVaoTable::new(),
        }
    }

    pub fn update_tables(&mut self, gamestate: &Game) {
        self.chunk_vaos
            .update_chunks(&gamestate.world, |chunk, world| { 
                generate_chunk_vertex_data(chunk, world.get_adjacent(chunk))
            });
        self.lava_vaos
            .update_chunks(&gamestate.world, |chunk, world| {
                generate_fluid_vertex_data(chunk, world.get_adjacent(chunk), world, 13)
            });
        self.water_vaos
            .update_chunks(&gamestate.world, |chunk, world| { 
                generate_fluid_vertex_data(chunk, world.get_adjacent(chunk), world, 12)
            });
    }

    pub fn clear(&mut self) {
        self.chunk_vaos.clear();
        self.lava_vaos.clear();
        self.water_vaos.clear();
    }
}
