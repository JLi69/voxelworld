use super::{ChunkData, FaceInfo, Int3};
use crate::gfx::face_data::{
    Face, DIAGONAL_FACE_1, DIAGONAL_FACE_1_REVERSED, DIAGONAL_FACE_2, DIAGONAL_FACE_2_REVERSED,
};
use crate::voxel::light::Light;
use crate::voxel::{Chunk, EMPTY_BLOCK};

fn add_face_plant(
    xyz: Int3,
    vert_data: &mut ChunkData,
    face: &Face,
    face_info: FaceInfo,
    light: Light,
) {
    let (x, y, z) = xyz;

    for i in 0..4 {
        let x = face[i * 3] + x as u8;
        let y = face[i * 3 + 1] + y as u8;
        let z = face[i * 3 + 2] + z as u8;
        vert_data.push(x);
        vert_data.push(y);
        vert_data.push(z);
        vert_data.push(face_info.block_texture_id);
        vert_data.push(face_info.face_id);
        vert_data.push(((light.r() as u8) << 4) | (light.skylight() as u8));
        vert_data.push(((light.b() as u8) << 4) | (light.g() as u8));
    }
}

pub fn add_block_vertices_plant(chunk: &Chunk, xyz: Int3, vert_data: &mut ChunkData) {
    let (x, y, z) = xyz;
    let blockid = chunk
        .get_block_relative(x as usize, y as usize, z as usize)
        .id;
    if blockid == EMPTY_BLOCK {
        return;
    }

    let facex = FaceInfo::new(blockid, 0);
    let facez = FaceInfo::new(blockid, 2);
    let light = chunk.get_light_relative(x as usize, y as usize, z as usize);
    add_face_plant(xyz, vert_data, &DIAGONAL_FACE_1, facex, light);
    add_face_plant(xyz, vert_data, &DIAGONAL_FACE_2, facez, light);
    add_face_plant(xyz, vert_data, &DIAGONAL_FACE_1_REVERSED, facex, light);
    add_face_plant(xyz, vert_data, &DIAGONAL_FACE_2_REVERSED, facez, light);
}
