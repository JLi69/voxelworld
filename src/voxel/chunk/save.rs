use std::fs::File;
use std::io::Write;
use std::path::Path;
use crate::voxel::Block;
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
