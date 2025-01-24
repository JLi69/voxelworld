use crate::gfx::buildchunk::{ChunkData, Int3};

pub fn increase_by_half(x: u8) -> u8 {
    x | (1 << 6)
}

pub fn decrease_by_half(x: u8) -> u8 {
    (x - 1) | (1 << 6)
}

//index = 0 -> x
//index = 1 -> y
//index = 2 -> z
fn half_positions(chunkdata: &mut ChunkData, pos: Int3, offset: i32, index: usize) {
    let (x, y, z) = pos;
    let pos = [x, y, z];
    let face_count = chunkdata.len() / (6 * 5);
    for i in 0..6 {
        let current_idx = (face_count - 1) * 6 * 5 + 5 * i + index;
        let coord = chunkdata[current_idx];
        if coord as i32 - pos[index] != offset {
            continue;
        }

        if offset == 0 {
            chunkdata[current_idx] = increase_by_half(chunkdata[current_idx]);
        } else if offset == 1 {
            chunkdata[current_idx] = decrease_by_half(chunkdata[current_idx]);
        }
    }
}

pub fn apply_slab_geometry(chunkdata: &mut ChunkData, pos: Int3, orientation: u8) {
    match orientation {
        //Up
        0 => half_positions(chunkdata, pos, 1, 1),
        //Right
        1 => half_positions(chunkdata, pos, 1, 0),
        //Front
        2 => half_positions(chunkdata, pos, 1, 2),
        //Down
        3 => half_positions(chunkdata, pos, 0, 1),
        //Left
        4 => half_positions(chunkdata, pos, 0, 0),
        //Back
        5 => half_positions(chunkdata, pos, 0, 2),
        _ => {}
    }
}
