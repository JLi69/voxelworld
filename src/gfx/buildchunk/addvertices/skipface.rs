use crate::{voxel::{orientation_to_normal, Block, EMPTY_BLOCK}, gfx::buildchunk::Int3};
use cgmath::{InnerSpace, Vector3};

fn show_face_default(adj_block: Block) -> bool {
    adj_block.id == EMPTY_BLOCK || adj_block.transparent()
}

fn show_face_trans(block: Block, adj_block: Block) -> bool {
    let id = block.id;
    let show_face = (adj_block.transparent() && adj_block.id != id)
        || (adj_block.transparent() && adj_block.id == id && !adj_block.can_connect());
    show_face || adj_block.id == EMPTY_BLOCK 
}

fn skip_block_face(block: Block, adj_block: Block, offset: Int3, show_face_fn: fn(Block, Block) -> bool) -> bool {
    let (offx, offy, offz) = offset;
    let diff = Vector3::new(offx, offy, offz);
    let adj_norm = orientation_to_normal(adj_block.orientation());
    if show_face_fn(block, adj_block) {
        return false;
    }
    match adj_block.shape() {
        //Full block
        0 => !show_face_fn(block, adj_block),
        //Slab
        1 => adj_norm == diff,
        //Stair
        2 => { 
            let y = if adj_block.reflection() == 0 {
                orientation_to_normal(0)
            } else {
                orientation_to_normal(3)
            };
            adj_norm == diff || y == diff
        }
        _ => false,
    }
}

fn skip_slab(norm: Vector3<i32>, adj_norm: Vector3<i32>, diff: Vector3<i32>) -> bool {
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

fn skip_slab_face(block: Block, adj_block: Block, offset: Int3, show_face_fn: fn(Block, Block) -> bool) -> bool {
    if show_face_fn(block, adj_block) {
        return false;
    }

    let orientation = block.orientation();
    let (offx, offy, offz) = offset;
    let diff = Vector3::new(offx, offy, offz);
    let norm = orientation_to_normal(orientation);
    let y = if adj_block.reflection() == 0 {
        orientation_to_normal(0)
    } else {
        orientation_to_normal(3)
    };
    let adj_norm = orientation_to_normal(adj_block.orientation()); 

    match adj_block.shape() {
        //Full block
        0 => {
            let dot = norm.dot(diff);
            dot == -1 || dot == 0
        }
        //Slab
        1 => skip_slab(norm, adj_norm, diff),
        //Stair
        2 => skip_slab(norm, adj_norm, diff) || skip_slab(norm, y, diff),
        _ => false,
    }
}

fn skip(block: Block, adj_block: Block, offset: Int3, show_face_fn: fn(Block, Block) -> bool) -> bool {
    match block.shape() {
        //Full block
        0 => skip_block_face(block, adj_block, offset, show_face_fn),
        //Slab
        1 => skip_slab_face(block, adj_block, offset, show_face_fn),
        //Stair
        2 => {
            let mut b = block;
            if b.reflection() == 0 {
                b.set_orientation(0);
            } else {
                b.set_orientation(3);
            }
            skip_slab_face(b, adj_block, offset, show_face_fn)
        }
        _ => false,
    }
}

pub fn skip_face(block: Block, adj_block: Block, offset: Int3) -> bool {
    skip(block, adj_block, offset, |_, adj_block| show_face_default(adj_block))
}

pub fn skip_face_trans(block: Block, adj_block: Block, offset: Int3) -> bool {
    skip(block, adj_block, offset, show_face_trans)
}
