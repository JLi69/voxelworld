use super::rotate_orientation;
use crate::{
    gfx::buildchunk::Int3,
    voxel::{orientation_to_normal, Block, EMPTY_BLOCK},
};
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

fn skip_block_face(
    block: Block,
    adj_block: Block,
    offset: Int3,
    show_face_fn: fn(Block, Block) -> bool,
) -> bool {
    if show_face_fn(block, adj_block) {
        return false;
    }

    let (offx, offy, offz) = offset;
    let diff = Vector3::new(offx, offy, offz);
    let adj_norm = orientation_to_normal(adj_block.orientation());
    let y = if adj_block.reflection() == 0 {
        orientation_to_normal(0)
    } else {
        orientation_to_normal(3)
    };
    match adj_block.shape() {
        //Full block
        0 => !show_face_fn(block, adj_block),
        //Slab
        1 => adj_norm == diff,
        //Stair
        2 => adj_norm == diff || y == diff,
        3 => y == diff,
        4 => {
            let rotated = rotate_orientation(adj_block.orientation());
            let rotated_norm = orientation_to_normal(rotated);
            adj_norm == diff || y == diff || rotated_norm == diff
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

fn skip_slab_face(
    block: Block,
    adj_block: Block,
    offset: Int3,
    show_face_fn: fn(Block, Block) -> bool,
) -> bool {
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
        3 => {
            let rotated = rotate_orientation(adj_block.orientation());
            let norm_rotated = orientation_to_normal(rotated);
            let skip_top = skip_slab(norm, adj_norm, diff) && skip_slab(norm, norm_rotated, diff);
            skip_slab(norm, y, diff) || skip_top
        }
        4 => {
            let rotated = rotate_orientation(adj_block.orientation());
            let rotated_norm = orientation_to_normal(rotated);
            skip_slab(norm, adj_norm, diff)
                || skip_slab(norm, y, diff)
                || skip_slab(norm, rotated_norm, diff)
        }
        _ => false,
    }
}

fn skip(
    block: Block,
    adj_block: Block,
    offset: Int3,
    show_face_fn: fn(Block, Block) -> bool,
) -> bool {
    match block.shape() {
        //Full block
        0 => skip_block_face(block, adj_block, offset, show_face_fn),
        //Slab
        1 => skip_slab_face(block, adj_block, offset, show_face_fn),
        //Stair
        2..=4 => {
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
    skip(block, adj_block, offset, |_, adj_block| {
        show_face_default(adj_block)
    })
}

pub fn skip_face_trans(block: Block, adj_block: Block, offset: Int3) -> bool {
    skip(block, adj_block, offset, show_face_trans)
}
