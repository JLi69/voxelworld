mod fluid;
mod furnace;
mod grass;
mod log;
mod nonvoxel;
mod plant;
mod slab;
mod transparent;

use super::{ChunkData, Int3};
use crate::gfx::face_data::{
    Face, BACK_FACE, BOTTOM_FACE, FRONT_FACE, LEFT_FACE, RIGHT_FACE, TOP_FACE,
};
use crate::voxel::{orientation_to_normal, out_of_bounds, wrap_coord, Block, Chunk, EMPTY_BLOCK};
use cgmath::{InnerSpace, Vector3};
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

fn skip_block_face(adj_block: Block, offset: Int3) -> bool {
    let (offx, offy, offz) = offset;
    match adj_block.shape() {
        //Full block
        0 => adj_block.id != EMPTY_BLOCK && !adj_block.transparent(),
        //Slab
        1 => {
            if adj_block.id == EMPTY_BLOCK || adj_block.transparent() {
                return false;
            }

            let adj_norm = orientation_to_normal(adj_block.orientation());
            let diff = Vector3::new(offx, offy, offz);
            adj_norm == diff
        }
        _ => false,
    }
}

fn skip_slab_face(orientation: u8, adj_block: Block, offset: Int3) -> bool {
    let (offx, offy, offz) = offset;
    match adj_block.shape() {
        //Full block
        0 => {
            if adj_block.id == EMPTY_BLOCK || adj_block.transparent() {
                return false;
            }

            let norm = orientation_to_normal(orientation);
            let diff = Vector3::new(offx, offy, offz);
            let dot = norm.dot(diff);
            dot == -1 || dot == 0
        }
        //Slab
        1 => {
            if adj_block.id == EMPTY_BLOCK || adj_block.transparent() {
                return false;
            }

            let norm = orientation_to_normal(orientation);
            let adj_norm = orientation_to_normal(adj_block.orientation());
            let diff = Vector3::new(offx, offy, offz);
            let dot = norm.dot(diff);
            if dot == 0 {
                norm == adj_norm || adj_norm == diff
            } else if dot == 1 {
                false
            } else if dot == -1 {
                adj_norm == diff
            } else {
                false
            }
        }
        _ => false,
    }
}

fn skip_face(block: Block, adj_block: Block, offset: Int3) -> bool {
    match block.shape() {
        //Full block
        0 => skip_block_face(adj_block, offset),
        //Slab
        1 => skip_slab_face(block.orientation(), adj_block, offset),
        _ => false,
    }
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
    let (offx, offy, offz) = offset;
    let block = chunk.get_block_relative(x as usize, y as usize, z as usize);

    let adj_x = wrap_coord(x + offx) as usize;
    let adj_y = wrap_coord(y + offy) as usize;
    let adj_z = wrap_coord(z + offz) as usize;
    if let Some(adj_chunk) = adj_chunk {
        let adj_block = adj_chunk.get_block_relative(adj_x, adj_y, adj_z);
        if out_of_bounds(x, y, z, offx, offy, offz) && skip_face(block, adj_block, offset) {
            return;
        }
    }

    if adj_chunk.is_none() && out_of_bounds(x, y, z, offx, offy, offz) {
        return;
    }

    let adj_x = (x + offx) as usize;
    let adj_y = (y + offy) as usize;
    let adj_z = (z + offz) as usize;
    let adj_block = chunk.get_block_relative(adj_x, adj_y, adj_z);
    if skip_face(block, adj_block, offset) {
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

    slab::apply_slab_geometry(vert_data, xyz, block.shape(), block.orientation());
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
