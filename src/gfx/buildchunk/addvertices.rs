mod fluid;
mod furnace;
mod grass;
mod log;
mod nonvoxel;
mod plant;
mod slab;
mod transparent;
mod skipface;

use super::{ChunkData, Int3};
use crate::gfx::face_data::{
    Face, BACK_FACE, BOTTOM_FACE, FRONT_FACE, LEFT_FACE, RIGHT_FACE, TOP_FACE,
};
use crate::voxel::{out_of_bounds, wrap_coord, Block, Chunk, EMPTY_BLOCK};
use skipface::skip_face;
pub use fluid::add_fluid_vertices;
pub use furnace::add_block_vertices_furnace_rotated;
pub use grass::add_block_vertices_grass;
pub use log::add_block_vertices_log;
pub use nonvoxel::add_nonvoxel_vertices;
pub use plant::add_block_vertices_plant;
pub use transparent::add_block_vertices_trans;

#[derive(Copy, Clone)]
struct FaceInfo {
    face_id: u8,
    block_texture_id: u8,
}

impl FaceInfo {
    fn new(blocki: u8, facei: u8) -> Self {
        Self {
            face_id: facei,
            block_texture_id: blocki,
        }
    }
}

fn apply_geometry(
    block: Block,
    xyz: Int3,
    vert_data: &mut ChunkData,
) {
    match block.shape() {
        1 => {
            slab::apply_slab_geometry(vert_data, xyz, block.orientation());
        }
        2 => {
            if block.reflection() == 0 {
                slab::apply_slab_geometry(vert_data, xyz, 0);
            } else {
                slab::apply_slab_geometry(vert_data, xyz, 3);
            }
        }
        _ => {}
    }
}

fn add_stair_geometry(
    block: Block,
    adj_block: Option<Block>,
    xyz: Int3,
    offset: Int3,
    face: &Face,
    face_info: FaceInfo,
    skip_face_fn: fn(block: Block, adj_block: Block, offset: Int3) -> bool,
) -> ChunkData {
    let (x, y, z) = xyz;
    let mut vert_data = vec![];
    match block.shape() {
        2 => {
            if let Some(adj_block) = adj_block {
                if skip_face_fn(block, adj_block, offset) {
                    return vert_data;
                }
            } else {
                return vert_data;
            }
            
            for i in 0..6 {
                let x = face[i * 3] + x as u8;
                let y = face[i * 3 + 1] + y as u8;
                let z = face[i * 3 + 2] + z as u8;
                vert_data.push(x);
                vert_data.push(y);
                vert_data.push(z);
                vert_data.push(face_info.block_texture_id);
                vert_data.push(face_info.face_id);
            }
            if block.reflection() == 0 {
                slab::apply_slab_geometry(&mut vert_data, xyz, 3);
            } else {
                slab::apply_slab_geometry(&mut vert_data, xyz, 0);
            }
            slab::apply_slab_geometry(&mut vert_data, xyz, block.orientation());
        }
        _ => {}
    }
    vert_data
}

fn get_adj_block(chunk: &Chunk, adj_chunk: Option<&Chunk>, xyz: Int3, offset: Int3) -> Option<Block> {
    let (x, y, z) = xyz;
    let (offx, offy, offz) = offset;

    if let Some(adj_chunk) = adj_chunk {
        let adj_x = wrap_coord(x + offx) as usize;
        let adj_y = wrap_coord(y + offy) as usize;
        let adj_z = wrap_coord(z + offz) as usize;
        let adj_block = adj_chunk.get_block_relative(adj_x, adj_y, adj_z);
        if out_of_bounds(x, y, z, offx, offy, offz) {
            return Some(adj_block);
        }
    }

    if adj_chunk.is_none() && out_of_bounds(x, y, z, offx, offy, offz) {
        return None;
    }

    let adj_x = (x + offx) as usize;
    let adj_y = (y + offy) as usize;
    let adj_z = (z + offz) as usize;
    let adj_block = chunk.get_block_relative(adj_x, adj_y, adj_z);
    Some(adj_block)
}

fn add_face(
    chunk: &Chunk,
    adj_chunk: Option<&Chunk>,
    xyz: Int3,
    offset: Int3,
    vert_data: &mut ChunkData,
    face: &Face,
    face_info: FaceInfo,
) {
    let (x, y, z) = xyz;
    let block = chunk.get_block_relative(x as usize, y as usize, z as usize);

    let adj_block = get_adj_block(chunk, adj_chunk, xyz, offset);
    if let Some(adj_block) = adj_block {
        if skip_face(block, adj_block, offset) {
            let stair = add_stair_geometry(block, Some(adj_block), xyz, offset, face, face_info, skip_face);
            vert_data.extend(stair);
            return;
        }
    } else {
        let stair = add_stair_geometry(block, adj_block, xyz, offset, face, face_info, skip_face);
        vert_data.extend(stair);
        return;
    }

    for i in 0..6 {
        let x = face[i * 3] + x as u8;
        let y = face[i * 3 + 1] + y as u8;
        let z = face[i * 3 + 2] + z as u8;
        vert_data.push(x);
        vert_data.push(y);
        vert_data.push(z);
        vert_data.push(face_info.block_texture_id);
        vert_data.push(face_info.face_id);
    }

    apply_geometry(block, xyz, vert_data);
    let stair = add_stair_geometry(block, adj_block, xyz, offset, face, face_info, skip_face);
    vert_data.extend(stair);
}

//Default function for adding block vertices, all faces have the same texture
pub fn add_block_vertices_default(
    chunk: &Chunk,
    adj_chunks: [Option<&Chunk>; 6],
    xyz: Int3,
    vert_data: &mut ChunkData,
) {
    let (x, y, z) = xyz;
    let block = chunk.get_block_relative(x as usize, y as usize, z as usize);
    if block.id == EMPTY_BLOCK {
        return;
    }

    let facex = FaceInfo::new(block.id, 0);
    let facey = FaceInfo::new(block.id, 1);
    let facez = FaceInfo::new(block.id, 2);

    #[rustfmt::skip]
    add_face(chunk, adj_chunks[0], xyz, (0, 1, 0), vert_data, &TOP_FACE, facey);
    #[rustfmt::skip]
    add_face(chunk, adj_chunks[1], xyz, (0, -1, 0), vert_data, &BOTTOM_FACE, facey);
    #[rustfmt::skip]
    add_face(chunk, adj_chunks[2], xyz, (-1, 0, 0), vert_data, &LEFT_FACE, facex);
    #[rustfmt::skip]
    add_face(chunk, adj_chunks[3], xyz, (1, 0, 0), vert_data, &RIGHT_FACE, facex);
    #[rustfmt::skip]
    add_face(chunk, adj_chunks[4], xyz, (0, 0, -1), vert_data, &FRONT_FACE, facez);
    #[rustfmt::skip]
    add_face(chunk, adj_chunks[5], xyz, (0, 0, 1), vert_data, &BACK_FACE, facez);
}

//Adds front face
pub fn add_block_vertices_flat(chunk: &Chunk, xyz: Int3, vert_data: &mut ChunkData) {
    let (x, y, z) = xyz;
    let blockid = chunk
        .get_block_relative(x as usize, y as usize, z as usize)
        .id;
    if blockid == EMPTY_BLOCK {
        return;
    }

    let facex = FaceInfo::new(blockid, 0);
    #[rustfmt::skip]
    add_face(chunk, None, xyz, (-1, 0, 0), vert_data, &LEFT_FACE, facex);
}
