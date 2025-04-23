use super::buildchunk::Indices;
use super::display::{get_sky_brightness, get_skycolor};
use super::fluid::generate_fluid_vertex_data;
use super::frustum::Frustum;
use super::nonvoxel::generate_non_voxel_vertex_data;
use super::{generate_chunk_vertex_data, ChunkData};
use crate::assets::shader::ShaderProgram;
use crate::game::inventory::Item;
use crate::game::physics::Hitbox;
use crate::game::Game;
use crate::voxel::{world_to_chunk_position, wrap_coord, Chunk, ChunkPos, World, CHUNK_SIZE_I32};
use crate::CHUNK_SIZE_F32;
use cgmath::Vector3;
use std::collections::{HashMap, VecDeque};
use std::mem::size_of;
use std::os::raw::c_void;

//Set fog color
pub fn set_fog(gamestate: &Game, shader: &ShaderProgram, skycolor: (f32, f32, f32)) {
    if gamestate.player.head_intersection(&gamestate.world, 12) {
        //Water
        shader.uniform_float("fogdist", -CHUNK_SIZE_F32 / 3.0);
        shader.uniform_float("fogstrength", 1.0 / CHUNK_SIZE_F32);
        shader.uniform_vec4f("fogcolor", 0.16, 0.41, 0.51, 1.0);
    } else if gamestate.player.head_intersection(&gamestate.world, 13) {
        //Lava
        shader.uniform_float("fogdist", -1.8 / 8.0);
        shader.uniform_float("fogstrength", 1.0 / 1.8);
        shader.uniform_vec4f("fogcolor", 1.0, 0.3, 0.0, 1.0);
    } else {
        let (sr, sg, sb) = skycolor;
        //Normal
        let range = gamestate.world.get_range() as f32 * CHUNK_SIZE_F32;
        let dist = range * 0.7;
        shader.uniform_float("fogdist", dist);
        shader.uniform_float("fogstrength", 1.0 / (range * 0.2));
        shader.uniform_vec4f("fogcolor", sr, sg, sb, 1.0);
    }
}

//Set dynamic lighting based on what the player is holding
pub fn set_dyn_light(gamestate: &Game, shader: &ShaderProgram) {
    if let Item::BlockItem(b, _) = gamestate.player.hotbar.get_selected() {
        if let Some(src) = b.light_src() {
            let (r, g, b) = src.rgb_f32();
            shader.uniform_vec3f("lightcolor", r, g, b);
        } else {
            shader.uniform_vec3f("lightcolor", 0.0, 0.0, 0.0);
        }
    } else {
        shader.uniform_vec3f("lightcolor", 0.0, 0.0, 0.0);
    }
}

const BUF_COUNT: usize = 3;

pub struct ChunkVao {
    id: u32,
    buffers: [u32; BUF_COUNT],
    vert_count: i32,
}

impl ChunkVao {
    pub fn generate_new(chunkdata: &ChunkData, indices: &Indices, values_per_vert: i32) -> Self {
        let mut vao = Self {
            id: 0,
            buffers: [0; BUF_COUNT],
            vert_count: 0,
        };

        unsafe {
            gl::GenVertexArrays(1, &mut vao.id);
            gl::GenBuffers(BUF_COUNT as i32, &mut vao.buffers[0]);
        }

        send_chunk_data_to_vao(&vao, indices, chunkdata, values_per_vert);
        vao.vert_count = indices.len() as i32;

        vao
    }

    pub fn draw(&self) {
        unsafe {
            gl::BindVertexArray(self.id);
            gl::DrawElements(
                gl::TRIANGLES,
                self.vert_count,
                gl::UNSIGNED_INT,
                std::ptr::null(),
            );
        }
    }

    pub fn delete(&self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.id);
            gl::DeleteBuffers(BUF_COUNT as i32, &self.buffers[0]);
        }
    }
}

//Send chunk vertex data to a buffer and vao
fn send_chunk_data_to_vao(
    chunkvao: &ChunkVao,
    indices: &Indices,
    chunkdata: &ChunkData,
    values_per_vert: i32,
) {
    if chunkdata.is_empty() || indices.is_empty() {
        return;
    }

    assert!(values_per_vert > 4);

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
            size_of::<u8>() as i32 * values_per_vert,
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
            values_per_vert - 4,
            gl::UNSIGNED_BYTE,
            size_of::<u8>() as i32 * values_per_vert,
            (size_of::<u8>() * 4) as *const c_void,
        );
        gl::EnableVertexAttribArray(1);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, chunkvao.buffers[2]);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            (indices.len() * size_of::<u32>()) as isize,
            &indices[0] as *const u32 as *const c_void,
            gl::STATIC_DRAW,
        );
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
        gen_verts: fn(&Chunk, &World) -> (ChunkData, Indices, i32),
    ) {
        let top = self.to_update.pop_front();
        if let Some(pos) = top {
            if !world.chunks.contains_key(&pos) {
                return;
            }

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
                    let (chunkdata, indices, vals_per_vert) = gen_verts(chunk, world);
                    send_chunk_data_to_vao(&vao, &indices, &chunkdata, vals_per_vert);
                    vao.vert_count = indices.len() as i32;
                    self.vaos.insert((chunkpos.x, chunkpos.y, chunkpos.z), vao);
                }
            }
        }
    }

    pub fn update_chunks(
        &mut self,
        world: &World,
        maxtime: f32,
        gen_verts: fn(&Chunk, &World) -> (ChunkData, Indices, i32),
    ) {
        let start = std::time::Instant::now();
        let mut total_time = 0.0;
        while total_time < maxtime && !self.to_update.is_empty() {
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
        gen_verts: fn(&Chunk, &World) -> (ChunkData, Indices, i32),
    ) {
        let mut vaos = vec![0; world.chunks.len()];
        let mut buffers = vec![0; world.chunks.len() * BUF_COUNT];

        unsafe {
            gl::GenVertexArrays(vaos.len() as i32, &mut vaos[0]);
            gl::GenBuffers(buffers.len() as i32, &mut buffers[0]);
        }

        for (i, chunk) in world.chunks.values().enumerate() {
            let chunkpos = chunk.get_chunk_pos();
            let (chunkdata, indices, vals_per_vert) = gen_verts(chunk, world);
            let chunkvao = ChunkVao {
                id: vaos[i],
                buffers: [
                    buffers[i * BUF_COUNT],
                    buffers[i * BUF_COUNT + 1],
                    buffers[i * BUF_COUNT + 2],
                ],
                vert_count: indices.len() as i32,
            };
            send_chunk_data_to_vao(&chunkvao, &indices, &chunkdata, vals_per_vert);
            self.vaos
                .insert((chunkpos.x, chunkpos.y, chunkpos.z), chunkvao);
        }
    }

    //Update chunk buffer data
    fn update_chunk_vao(
        &mut self,
        chunk: Option<&Chunk>,
        world: &World,
        gen_verts: fn(&Chunk, &World) -> (ChunkData, Indices, i32),
    ) {
        if let Some(chunk) = chunk {
            let chunkpos = chunk.get_chunk_pos();
            let (chunkdata, indices, vals_per_vert) = gen_verts(chunk, world);
            let chunkvao = self.vaos.get_mut(&(chunkpos.x, chunkpos.y, chunkpos.z));
            if let Some(chunkvao) = chunkvao {
                chunkvao.vert_count = indices.len() as i32;
                send_chunk_data_to_vao(chunkvao, &indices, &chunkdata, vals_per_vert);
            }
        }
    }

    //Update any adjacent chunks that might also be affected by a block update
    fn update_adjacent(
        &mut self,
        x: i32,
        y: i32,
        z: i32,
        world: &World,
        gen_verts: fn(&Chunk, &World) -> (ChunkData, Indices, i32),
    ) {
        let (chunkx, chunky, chunkz) = world_to_chunk_position(x, y, z);
        let x = wrap_coord(x % CHUNK_SIZE_I32);
        let y = wrap_coord(y % CHUNK_SIZE_I32);
        let z = wrap_coord(z % CHUNK_SIZE_I32);
        for dx in -1..=1 {
            for dy in -1..=1 {
                for dz in -1..=1 {
                    if dx == 0 && dy == 0 && dz == 0 {
                        continue;
                    }

                    if (dx == -1 && x != 0) || (dx == 1 && x != CHUNK_SIZE_I32 - 1) {
                        continue;
                    }

                    if (dy == -1 && y != 0) || (dy == 1 && y != CHUNK_SIZE_I32 - 1) {
                        continue;
                    }

                    if (dz == -1 && z != 0) || (dz == 1 && z != CHUNK_SIZE_I32 - 1) {
                        continue;
                    }

                    let chunk = world.get_chunk(chunkx + dx, chunky + dy, chunkz + dz);
                    self.update_chunk_vao(chunk, world, gen_verts);
                }
            }
        }
    }

    //Updates a single chunk and if necessary, also updates the adjacent chunks
    pub fn update_chunk_with_adj(
        &mut self,
        x: i32,
        y: i32,
        z: i32,
        world: &World,
        gen_verts: fn(&Chunk, &World) -> (ChunkData, Indices, i32),
    ) {
        let (chunkx, chunky, chunkz) = world_to_chunk_position(x, y, z);
        let chunk = world.get_chunk(chunkx, chunky, chunkz);
        self.update_chunk_vao(chunk, world, gen_verts);
        if chunk.is_some() {
            self.update_adjacent(x, y, z, world, gen_verts);
        }
    }

    //Ignores the corners
    fn update_adjacent_fast(
        &mut self,
        x: i32,
        y: i32,
        z: i32,
        world: &World,
        gen_verts: fn(&Chunk, &World) -> (ChunkData, Indices, i32),
    ) {
        let (chunkx, chunky, chunkz) = world_to_chunk_position(x, y, z);
        let x = wrap_coord(x % CHUNK_SIZE_I32);
        let y = wrap_coord(y % CHUNK_SIZE_I32);
        let z = wrap_coord(z % CHUNK_SIZE_I32);
        for dx in -1i32..=1 {
            for dy in -1i32..=1 {
                for dz in -1i32..=1 {
                    if dx.abs() == 1 && dy.abs() == 1 && dz.abs() == 1 {
                        continue;
                    }

                    if dx == 0 && dy == 0 && dz == 0 {
                        continue;
                    }

                    if (dx == -1 && x != 0) || (dx == 1 && x != CHUNK_SIZE_I32 - 1) {
                        continue;
                    }

                    if (dy == -1 && y != 0) || (dy == 1 && y != CHUNK_SIZE_I32 - 1) {
                        continue;
                    }

                    if (dz == -1 && z != 0) || (dz == 1 && z != CHUNK_SIZE_I32 - 1) {
                        continue;
                    }

                    let chunk = world.get_chunk(chunkx + dx, chunky + dy, chunkz + dz);
                    self.update_chunk_vao(chunk, world, gen_verts);
                }
            }
        }
    }

    pub fn update_chunk_with_adj_fast(
        &mut self,
        x: i32,
        y: i32,
        z: i32,
        world: &World,
        gen_verts: fn(&Chunk, &World) -> (ChunkData, Indices, i32),
    ) {
        let (chunkx, chunky, chunkz) = world_to_chunk_position(x, y, z);
        let chunk = world.get_chunk(chunkx, chunky, chunkz);
        self.update_chunk_vao(chunk, world, gen_verts);
        if chunk.is_some() {
            self.update_adjacent_fast(x, y, z, world, gen_verts);
        }
    }

    //Displays all the chunk vaos
    pub fn display_chunks(&self, gamestate: &Game, shaderid: &str) -> u32 {
        if gamestate.invert_backface_culling {
            unsafe {
                gl::CullFace(gl::FRONT);
            }
        }

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
        chunkshader.uniform_float("skybrightness", get_sky_brightness(gamestate.world.time));

        //Set fog color
        set_fog(gamestate, &chunkshader, get_skycolor(gamestate.world.time));
        //Dynamic lighting
        set_dyn_light(gamestate, &chunkshader);

        let mut drawn_count = 0;
        let (centerx, centery, centerz) = gamestate.world.get_center();
        for ((chunkx, chunky, chunkz), vao) in &self.vaos {
            if vao.vert_count == 0 {
                continue;
            }

            //Cull out chunks that are too far away
            let dist2 = (*chunkx - centerx) * (*chunkx - centerx)
                + (*chunky - centery) * (*chunky - centery)
                + (*chunkz - centerz) * (*chunkz - centerz);
            let range = gamestate.world.get_range(); 
            if dist2 > (range + 1) * (range + 1) {
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
                gl::DrawElements(
                    gl::TRIANGLES,
                    vao.vert_count,
                    gl::UNSIGNED_INT,
                    std::ptr::null(),
                );
            }
        }

        if gamestate.invert_backface_culling {
            unsafe {
                gl::CullFace(gl::BACK);
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
            .update_chunk_with_adj_fast(x, y, z, world, |chunk, world| {
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
        chunks
            .non_voxel_vaos
            .update_chunk_with_adj_fast(x, y, z, world, |chunk, _| {
                generate_non_voxel_vertex_data(chunk)
            });
    }
}

pub struct ChunkTables {
    pub chunk_vaos: ChunkVaoTable,
    pub lava_vaos: ChunkVaoTable,
    pub water_vaos: ChunkVaoTable,
    pub non_voxel_vaos: ChunkVaoTable,
}

impl ChunkTables {
    pub fn new() -> Self {
        Self {
            chunk_vaos: ChunkVaoTable::new(),
            lava_vaos: ChunkVaoTable::new(),
            water_vaos: ChunkVaoTable::new(),
            non_voxel_vaos: ChunkVaoTable::new(),
        }
    }

    pub fn init_tables(&mut self, world: &World) {
        self.chunk_vaos.generate_chunk_vaos(world, |chunk, world| {
            generate_chunk_vertex_data(chunk, world.get_adjacent(chunk))
        });
        self.lava_vaos.generate_chunk_vaos(world, |chunk, world| {
            generate_fluid_vertex_data(chunk, world.get_adjacent(chunk), world, 13)
        });
        self.water_vaos.generate_chunk_vaos(world, |chunk, world| {
            generate_fluid_vertex_data(chunk, world.get_adjacent(chunk), world, 12)
        });
        self.non_voxel_vaos
            .generate_chunk_vaos(world, |chunk, _| generate_non_voxel_vertex_data(chunk));
    }

    pub fn update_tables(&mut self, gamestate: &Game) {
        self.chunk_vaos
            .update_chunks(&gamestate.world, 0.0004, |chunk, world| {
                generate_chunk_vertex_data(chunk, world.get_adjacent(chunk))
            });
        self.lava_vaos
            .update_chunks(&gamestate.world, 0.0001, |chunk, world| {
                generate_fluid_vertex_data(chunk, world.get_adjacent(chunk), world, 13)
            });
        self.water_vaos
            .update_chunks(&gamestate.world, 0.0001, |chunk, world| {
                generate_fluid_vertex_data(chunk, world.get_adjacent(chunk), world, 12)
            });
        self.non_voxel_vaos
            .update_chunks(&gamestate.world, 0.0001, |chunk, _| {
                generate_non_voxel_vertex_data(chunk)
            });
    }

    pub fn update_table(&mut self, world: &World, x: i32, y: i32, z: i32) {
        self.chunk_vaos
            .update_chunk_vao(world.get_chunk(x, y, z), world, |chunk, world| {
                generate_chunk_vertex_data(chunk, world.get_adjacent(chunk))
            });
        self.lava_vaos
            .update_chunk_vao(world.get_chunk(x, y, z), world, |chunk, world| {
                generate_fluid_vertex_data(chunk, world.get_adjacent(chunk), world, 13)
            });
        self.water_vaos
            .update_chunk_vao(world.get_chunk(x, y, z), world, |chunk, world| {
                generate_fluid_vertex_data(chunk, world.get_adjacent(chunk), world, 12)
            });
        self.non_voxel_vaos
            .update_chunk_vao(world.get_chunk(x, y, z), world, |chunk, _| {
                generate_non_voxel_vertex_data(chunk)
            });
    }

    pub fn clear(&mut self) {
        self.chunk_vaos.clear();
        self.lava_vaos.clear();
        self.water_vaos.clear();
        self.non_voxel_vaos.clear();
    }
}
