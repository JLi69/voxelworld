use super::{add_face, ChunkData, FaceInfo, Int3};
use crate::gfx::face_data::{BACK_FACE, BOTTOM_FACE, FRONT_FACE, LEFT_FACE, RIGHT_FACE, TOP_FACE};
use crate::voxel::{Chunk, EMPTY_BLOCK};

//This is for adding vertices such that the resulting block is textured like a
//log (this function can also be used as for other blocks that are
//textured similarly I just didn't really have a good name for this)
//By default, the "blockid" will be used as the side textures
pub fn add_block_vertices_log(
    chunk: &Chunk,
    adj_chunks: [Option<&Chunk>; 6],
    xyz: Int3,
    vert_data: &mut ChunkData,
    faceytexture: u8,
    sidetexture_rotated: u8,
) {
    let (x, y, z) = xyz;
    let block = chunk.get_block_relative(x as usize, y as usize, z as usize);
    if block.id == EMPTY_BLOCK {
        return;
    }

    let (facex, facey, facez) = match block.orientation() % 3 {
        0 => (
            FaceInfo::new(block.id, 0),
            FaceInfo::new(faceytexture, 1),
            FaceInfo::new(block.id, 2),
        ),
        1 => (
            FaceInfo::new(faceytexture, 0),
            FaceInfo::new(block.id, 1),
            FaceInfo::new(sidetexture_rotated, 2),
        ),
        2 => (
            FaceInfo::new(sidetexture_rotated, 0),
            FaceInfo::new(sidetexture_rotated, 1),
            FaceInfo::new(faceytexture, 2),
        ),
        _ => (
            FaceInfo::new(block.id, 0),
            FaceInfo::new(faceytexture, 1),
            FaceInfo::new(block.id, 2),
        ),
    };

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
