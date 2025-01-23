use crate::gfx::buildchunk::{ChunkData, Int3};
use crate::gfx::models::{CUBE, CUBE_INDICES, CUBE_TEX_INDICES, TEX_COORDS, QUAD_INDICES};
use crate::voxel::{Block, Chunk};
use cgmath::{Deg, Matrix4, Vector2, Vector3, Vector4};

type Vert = Vector3<f32>;
type Vert4 = Vector4<f32>;
type Norm = Vector3<f32>;
type Tc = Vector2<f32>;
type BlockMesh = (Vec<Vert>, Vec<Tc>);

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

fn add_mesh_to_chunk(xyz: Int3, id: u8, vertices: &[Vert],  tc: &[Tc], vert_data: &mut ChunkData) {
    let (x, y, z) = xyz;
    for (i, v) in vertices.iter().enumerate() {
        let vx = v.x + x as f32;
        let vy = v.y + y as f32;
        let vz = v.z + z as f32;

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
        let tc = tc[i];
        let (tcx1, tcx2) = tc_to_u8(tc.x);
        let (tcy1, tcy2) = tc_to_u8(tc.y);

        vert_data.push(vertx | (fx1 << 6));
        vert_data.push(verty | (fy1 << 6));
        vert_data.push(vertz | (fz1 << 6));
        vert_data.push(id);
        vert_data.push(1);
        vert_data.push(fraction | (tcx1 << 6) | (tcy1 << 7));
        vert_data.push((tcy2 << 4) | tcx2);
    }
}

fn generate_mesh_vertices(data: &[f32], indices: &[u32]) -> Vec<Vert> {
    indices.iter()
        .map(|index| *index as usize)
        .map(|index| Vector3::new(data[index * 3], data[index * 3 + 1], data[index * 3 + 2]))
        .collect()
}

fn generate_mesh_normals(vertices: &[Vector3<f32>]) -> Vec<Norm> {
    vertices.iter()
        .enumerate()
        .map(|(i, _)| {
            let i = i / 3;
            let v1 = vertices[i * 3];
            let v2 = vertices[i * 3 + 1];
            let v3 = vertices[i * 3 + 2];
            cgmath::Vector3::cross(v3 - v1, v2 - v1) 
        })
        .collect()
}

fn generate_mesh_texcoords(tc: &[f32], indices: &[u32]) -> Vec<Tc> {
    indices.iter()
        .map(|index| *index as usize)
        .map(|index| Vector2::new(tc[index * 2], tc[index * 2 + 1]))
        .collect()
}

fn transform_tc<T>(texcoords: &[Tc], transform_func: T) -> Vec<Tc> where T: Fn(Tc, usize) -> Tc {
    texcoords.iter()
        .enumerate()
        .map(|(i, tc)| transform_func(*tc, i))
        .collect()
}

fn transform_vertices<T>(vertices: &[Vert], transform: T) -> Vec<Vert> where T: Fn(Vert4) -> Vert4 {
    vertices.iter()
        .map(|v| Vector4::new(v.x, v.y, v.z, 1.0))
        .map(transform)
        .map(|v| Vert::new(v.x, v.y, v.z))
        .collect()
}

//Returns (vertices, texture coordinates)
fn gen_torch_vertices(block: Block) -> BlockMesh {
    let vertices = generate_mesh_vertices(&CUBE, &CUBE_INDICES);
    let normals = generate_mesh_normals(&vertices);

    let texcoords = generate_mesh_texcoords(&TEX_COORDS, &CUBE_TEX_INDICES);
    let torch_tc = transform_tc(&texcoords, |tc, i| {
        let norm = normals[i];
        let mut tc = tc;
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
        tc
    });

    let torch_vertices = transform_vertices(&vertices, |v| {
        let mut transformed = v;
        transformed.x *= 1.0 / 8.0;
        transformed.z *= 1.0 / 8.0;
        transformed.y *= 10.0 / 16.0;

        const TORCH_ROTATION: f32 = 35.0;
        const TORCH_OFFSET: f32 = 6.0 / 16.0;
        transformed = match block.orientation() {
            //We add the extra degree to prevent the torch from
            //appearing too thin when placed on one side of a block
            1 => Matrix4::from_angle_z(Deg(-TORCH_ROTATION + 1.0)) * transformed,
            2 => Matrix4::from_angle_x(Deg(TORCH_ROTATION - 1.0)) * transformed,
            3 => Matrix4::from_angle_x(Deg(180.0)) * transformed,
            4 => Matrix4::from_angle_z(Deg(TORCH_ROTATION)) * transformed,
            5 => Matrix4::from_angle_x(Deg(-TORCH_ROTATION)) * transformed,
            _ => transformed,
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
        transformed.y -= 3.0 / 16.0;
        transformed += Vert4::new(0.5, 0.5, 0.5, 0.0);

        transformed
    });

    (torch_vertices, torch_tc)
}

fn gen_ladder_vertices(block: Block) -> BlockMesh {
    let vertices = generate_mesh_vertices(&CUBE, &QUAD_INDICES);
    let texcoords = generate_mesh_texcoords(&TEX_COORDS, &QUAD_INDICES);
    let ladder_vertices = transform_vertices(&vertices, |v| {
        let mut transformed = v;
        transformed = Matrix4::from_angle_x(Deg(90.0)) * transformed;
        match block.orientation() {
            1 => { 
                transformed = Matrix4::from_angle_y(Deg(270.0)) * transformed;
                transformed.x -= 15.0 / 16.0;
            }
            2 => {
                transformed = Matrix4::from_angle_y(Deg(180.0)) * transformed;
                transformed.z -= 15.0 / 16.0
            }
            4 => {
                transformed = Matrix4::from_angle_y(Deg(90.0)) * transformed;
                transformed.x += 15.0 / 16.0;
            }
            5 => {
                transformed.z += 15.0 / 16.0;
            }
            _ => {}
        }
        transformed  += Vert4::new(0.5, 0.5, 0.5, 0.0);
        transformed
    });
    (ladder_vertices, texcoords)
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

    let (vert, tc) = match block.id {
        //Torch
        71..=74 => gen_torch_vertices(block),
        //Ladder
        75 => gen_ladder_vertices(block),
        _ => (vec![], vec![])
    };

    add_mesh_to_chunk(xyz, block.id, &vert, &tc, vert_data);
}
