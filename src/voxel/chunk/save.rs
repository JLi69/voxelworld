use std::fs::File;
use std::io::Write;
use std::path::Path;
use crate::voxel::{Block, CHUNK_SIZE};
use crate::game::save::CHUNK_PATH;
use super::Chunk;

pub fn chunk_file_name(x: i32, y: i32, z: i32) -> String { 
    "chnk_".to_string() + 
        x.to_string().as_str() +
        "_" +
        y.to_string().as_str() +
        "_" +
        z.to_string().as_str()
}

impl Chunk {
    pub fn from_rle(x: i32, y: i32, z: i32, blocks: &[(u16, Block)]) -> Self {
        let mut chunk_blocks = Vec::with_capacity(CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE);
        for (count, block) in blocks {
            for _ in 0..*count {
                if chunk_blocks.len() >= CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE {
                    break;
                }
                chunk_blocks.push(*block);
            }
        }

        Self {
            blocks: chunk_blocks,
            ix: x,
            iy: y,
            iz: z,
        }
    }

    pub fn rle_encode(&self) -> Vec<(u16, Block)> {
        let mut data = vec![];

        let mut count = 0u16;
        let mut block = Block::new();
        for b in &self.blocks {
            if *b != block {
                if count != 0 {
                    data.push((count, block));
                }

                block = *b;
                count = 1;
                continue;
            }

            count += 1;
        }

        if count != 0 {
            data.push((count, block));
        }

        data
    }

    //Returns Err(path) if failed to save, Ok(()) otherwise
    pub fn save_chunk(&self, worldpath: &str) -> Result<(), String> {
        let chunk_dir_path = worldpath.to_string() + CHUNK_PATH;
        let chunk_path = chunk_dir_path.clone() + chunk_file_name(self.ix, self.iy, self.iz).as_str();
        if !Path::new(&chunk_dir_path).exists() {
            if let Err(msg) = std::fs::create_dir_all(&chunk_dir_path) {
                eprintln!("E: Failed to create chunk dir");
                eprintln!("{msg}");
                return Err(chunk_path);
            }
        }

        let rle = self.rle_encode();
        let mut counts = Vec::<u8>::with_capacity(rle.len() * size_of::<u16>());
        let mut blockids = Vec::<u8>::with_capacity(rle.len());
        let mut blockorientations = Vec::<u8>::with_capacity(rle.len());
        for (count, b) in rle {
            let count_bytes = count.to_be_bytes();
            counts.push(count_bytes[0]);
            counts.push(count_bytes[1]);
            blockids.push(b.id);
            blockorientations.push(b.orientation);
        }

        match File::create(&chunk_path) {
            Ok(mut file) => {
                let len = (counts.len() as u16 / size_of::<u16>() as u16).to_be_bytes();
                if let Err(msg) = file.write_all(&len) { 
                    eprintln!("Error when saving {}, {}, {}", self.ix, self.iy, self.iz);
                    eprintln!("E: {msg}");
                    return Err(chunk_path);
                }
                if let Err(msg) = file.write_all(&counts) { 
                    eprintln!("Error when saving {}, {}, {}", self.ix, self.iy, self.iz);
                    eprintln!("E: {msg}");
                    return Err(chunk_path);
                }
                if let Err(msg) = file.write_all(&blockids) {
                    eprintln!("Error when saving {}, {}, {}", self.ix, self.iy, self.iz);
                    eprintln!("E: {msg}");
                    return Err(chunk_path);
                }
                if let Err(msg) = file.write_all(&blockorientations) {
                    eprintln!("Error when saving {}, {}, {}", self.ix, self.iy, self.iz);
                    eprintln!("E: {msg}");
                    return Err(chunk_path);
                }
            }
            Err(msg) => {
                eprintln!("Failed to save chunk {}, {}, {}", self.ix, self.iy, self.iz);
                eprintln!("{msg}");
                return Err(chunk_path);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filename() {
        let filename = chunk_file_name(1, 2, 3);
        assert_eq!(filename, "chnk_1_2_3".to_string());
    }

    #[test]
    fn test_rle() {
        let mut testchunk = Chunk::new(0, 0, 0);
        testchunk.set_block(15, 15, 15, Block::new_id(1));
        let rle = testchunk.rle_encode();
        let chunk2 = Chunk::from_rle(0, 0, 0, &rle);
        assert!(!rle.is_empty());
        for (i, b) in chunk2.blocks.iter().enumerate() {
            assert_eq!(*b, testchunk.blocks[i]);
        }
    }

    #[test]
    fn test_rle_empty() {
        let testchunk = Chunk::new(0, 0, 0); 
        let rle = testchunk.rle_encode();
        assert_eq!(rle.len(), 0);
        let chunk2 = Chunk::from_rle(0, 0, 0, &rle);
        assert_eq!(chunk2.blocks.len(), 0);
        assert_eq!(testchunk.blocks.len(), chunk2.blocks.len());
    }
}
