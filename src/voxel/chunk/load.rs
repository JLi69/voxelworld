use crate::{game::save::CHUNK_PATH, voxel::Block};
use super::Chunk;
use std::{fs::File, io::Read};

fn read_bytes(bytes: &mut [u8], file: &mut File) {
    if let Err(msg) = file.read(bytes) {
        eprintln!("Error when loading chunk: {msg}");
    }
}

fn convert_bytes_to_blocks(block_counts_bytes: &[u8], block_bytes: &[u8]) -> Vec<(u16, Block)> {
    let mut blocks = vec![];

    let len = block_counts_bytes.len() / 2;
    for i in 0..len {
        let index = 2 * i;
        let count = ((block_counts_bytes[index] as u16) << 8) | block_counts_bytes[index + 1] as u16;
        let id = block_bytes[i];
        let orientation = block_bytes[i + len as usize];
        blocks.push((count, Block::new_id_orientation(id, orientation)));
    }
    
    blocks
}

impl Chunk {
    pub fn load_chunk(world_dir_path: &str, x: i32, y: i32, z: i32) -> Option<Self> {
        let path = 
            world_dir_path.to_string() + 
            CHUNK_PATH + 
            "chnk_" +
            x.to_string().as_str() +
            "_" +
            y.to_string().as_str() +
            "_" +
            z.to_string().as_str();
 
        match File::open(&path) {
            Ok(mut file) => {
                let mut len_bytes = [ 0u8; size_of::<u16>() ];
                read_bytes(&mut len_bytes, &mut file);
                let len = ((len_bytes[0] as u16) << 8) | (len_bytes[1] as u16);

                if len == 0 {
                    return Some(Chunk::new(x, y, z));
                }

                let mut block_counts_bytes = vec![0u8; len as usize * 2];
                read_bytes(&mut block_counts_bytes, &mut file);
                let mut block_bytes = vec![0u8; len as usize * 2];
                read_bytes(&mut block_bytes, &mut file);
                let blocks = convert_bytes_to_blocks(&block_counts_bytes, &block_bytes);
                Some(Chunk::from_rle(x, y, z, &blocks))
            }
            Err(_msg) => {
                None
            }
        }
    }
}
