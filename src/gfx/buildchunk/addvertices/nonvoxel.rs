use cgmath::{Vector3, Vector2, Matrix4, Deg, Vector4};
use crate::voxel::{Chunk, Block};
use crate::gfx::buildchunk::{ChunkData, Int3};
use crate::gfx::models::{CUBE, CUBE_INDICES, CUBE_TEX_INDICES, TEX_COORDS};

fn fraction(x: f32) -> f32 {
    if x < 0.0 {
        x.fract() + 1.0
    } else {
        x.fract()
    }
}

fn fract_to_u8(x: f32) -> (u8, u8) {
    let f = ((fraction(x) * 16.0).round() as u8).min(15);
    (f >> 2, f & 3)
}

fn tc_to_u8(tc: f32) -> (u8, u8) {
    if tc == 1.0 {
        (1, 0)
    } else {
        (0, (tc * 16.0).floor() as u8)
    }
}

fn add_vertices_torch(block: Block, xyz: Int3, vert_data: &mut ChunkData) {
    let (x, y, z) = xyz;
    let mut torch_vertices = vec![];
    for index in CUBE_INDICES {
        let index = index as usize;
        let v = Vector3::new(CUBE[index * 3], CUBE[index * 3 + 1], CUBE[index * 3 + 2]);
        torch_vertices.push(v);
    }

    let mut torch_tc = vec![];
    for (i, index) in CUBE_TEX_INDICES.iter().enumerate() {
        let i = i / 3;
        let norm = cgmath::Vector3::cross(
            torch_vertices[i * 3 + 1] - torch_vertices[i * 3],
            torch_vertices[i * 3 + 2] - torch_vertices[i * 3],
        ); 
    
        let index = *index as usize;
        let mut tc = Vector2::new(TEX_COORDS[index * 2], TEX_COORDS[index * 2 + 1]);
        
        tc.x -= 0.5;
        tc.x *= 1.0 / 8.0;
        tc.x += 0.5;
        if norm.y != 0.0 { 
            tc.y -= 10.0 / 16.0;
            tc.y *= 1.0 / 8.0;
            tc.y += 10.0 / 16.0;
        } else {
            tc.y *= 10.0 / 16.0;
        }

        torch_tc.push(tc);
    }

    //Apply vertex transformations here
    for v in &mut torch_vertices {
        v.x *= 1.0 / 8.0;
        v.z *= 1.0 / 8.0;
        v.y *= 10.0 / 16.0;

        let v4 = Vector4::new(v.x, v.y, v.z, 1.0);
        const TORCH_ROTATION: f32 = 35.0;
        const TORCH_OFFSET: f32 = 6.0 / 16.0;
        let mut transformed = match block.orientation() {
            //We add the extra degree to prevent the torch from 
            //appearing too thin when placed on one side of a block
            1 => Matrix4::from_angle_z(Deg(-TORCH_ROTATION + 1.0)) * v4,
            2 => Matrix4::from_angle_x(Deg(TORCH_ROTATION - 1.0)) * v4, 
            3 => Matrix4::from_angle_x(Deg(180.0)) * v4,
            4 => Matrix4::from_angle_z(Deg(TORCH_ROTATION)) * v4, 
            5 => Matrix4::from_angle_x(Deg(-TORCH_ROTATION)) * v4, 
            _ => v4,
        };
        match block.orientation() {
            1 => transformed.x -= TORCH_OFFSET,
            2 => transformed.z -= TORCH_OFFSET,
            3 => transformed.y += TORCH_OFFSET,
            4 => transformed.x += TORCH_OFFSET,
            5 => transformed.z += TORCH_OFFSET,
            _ => {}
        }
        if block.orientation() % 3 != 0 {
            transformed.y += 0.15;
        }
        *v = Vector3::new(transformed.x, transformed.y, transformed.z);

        v.y -= 3.0 / 16.0;
    }

    for (i, v) in torch_vertices.iter().enumerate() {
        let vx = v.x + 0.5 + x as f32;
        let vy = v.y + 0.5 + y as f32;
        let vz = v.z + 0.5 + z as f32;

        let vx = if vx < 0.0 {
            //To mark a value as being negative, have it exceed 33
            vx + 33.0 + 1.0
        } else {
            vx
        };

        let vz = if vz < 0.0 {
            //To mark a value as being negative, have it exceed 33
            vz + 33.0 + 1.0
        } else {
            vz
        };

        let (fx1, fx2) = fract_to_u8(vx);
        let (fy1, fy2) = fract_to_u8(vy);
        let (fz1, fz2) = fract_to_u8(vz);

        let vertx = vx as u8;
        let verty = vy as u8;
        let vertz = vz as u8;
        let fraction = (fz2 << 4) | (fy2 << 2) | fx2;
        let tc = torch_tc[i];
        let (tcx1, tcx2) = tc_to_u8(tc.x);
        let (tcy1, tcy2) = tc_to_u8(tc.y);

        vert_data.push(vertx | (fx1 << 6));
        vert_data.push(verty | (fy1 << 6));
        vert_data.push(vertz | (fz1 << 6));
        vert_data.push(block.id);
        vert_data.push(1);
        vert_data.push(fraction | (tcx1 << 6) | (tcy1 << 7));
        vert_data.push((tcy2 << 4) | tcx2);
    }
}

pub fn add_nonvoxel_vertices(
    chunk: &Chunk,
    xyz: Int3,
    vert_data: &mut ChunkData,
) {
    let (x, y, z) = xyz;
    let block = chunk.get_block_relative(x as usize, y as usize, z as usize);

    if !block.non_voxel_geometry() {
        return;
    }

    match block.id {
        //Torch
        71..=74 => add_vertices_torch(block, xyz, vert_data),
        _ => {}
    }
}
