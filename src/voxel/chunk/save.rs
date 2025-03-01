use super::Chunk;
use crate::voxel::{Block, CHUNK_SIZE};

impl Chunk {
    pub fn from_rle(x: i32, y: i32, z: i32, blocks: &[(u16, Block)]) -> Self {
        if blocks.is_empty() {
            return Self::new(x, y, z);
        }

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
            light: vec![],
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

    //Returns (len, counts, data)
    pub fn get_chunk_bytes(&self) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
        let rle = self.rle_encode();
        let mut counts = Vec::<u8>::with_capacity(rle.len() * size_of::<u16>());
        let mut blockdata = Vec::<u8>::with_capacity(rle.len());
        for (count, b) in rle {
            let count_bytes = count.to_be_bytes();
            counts.push(count_bytes[0]);
            counts.push(count_bytes[1]);
            blockdata.push(b.id);
            blockdata.push(b.geometry);
        }
        let len = (counts.len() as u16 / size_of::<u16>() as u16).to_be_bytes();

        (len.to_vec(), counts, blockdata)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
