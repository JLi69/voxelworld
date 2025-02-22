use std::collections::HashMap;

use crate::gfx::buildchunk::{ChunkData, Int3};
use crate::gfx::models::{CUBE, CUBE_INDICES, CUBE_TEX_INDICES, QUAD_INDICES, TEX_COORDS};
use crate::voxel::light::Light;
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

fn add_mesh_to_chunk(
    xyz: Int3,
    id: u8,
    vertices: &[Vert],
    tc: &[Tc],
    vert_data: &mut ChunkData,
    light: Light,
) {
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
        //Sky light and red channel
        vert_data.push(((light.r() as u8) << 4) | (light.skylight() as u8));
        vert_data.push(fraction | (tcx1 << 6) | (tcy1 << 7));
        vert_data.push((tcy2 << 4) | tcx2);
        //Green and blue channel
        vert_data.push(((light.b() as u8) << 4) | (light.g() as u8));
    }
}

fn generate_mesh_vertices(data: &[f32], indices: &[u32]) -> Vec<Vert> {
    indices
        .iter()
        .map(|index| *index as usize)
        .map(|index| Vector3::new(data[index * 3], data[index * 3 + 1], data[index * 3 + 2]))
        .collect()
}

fn generate_mesh_normals(vertices: &[Vector3<f32>]) -> Vec<Norm> {
    vertices
        .iter()
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
    indices
        .iter()
        .map(|index| *index as usize)
        .map(|index| Vector2::new(tc[index * 2], tc[index * 2 + 1]))
        .collect()
}

fn transform_tc<T>(texcoords: &[Tc], transform_func: T) -> Vec<Tc>
where
    T: Fn(Tc, usize) -> Tc,
{
    texcoords
        .iter()
        .enumerate()
        .map(|(i, tc)| transform_func(*tc, i))
        .collect()
}

fn transform_vertices<T>(vertices: &[Vert], transform: T) -> Vec<Vert>
where
    T: Fn(Vert4) -> Vert4,
{
    vertices
        .iter()
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
        transformed += Vert4::new(0.5, 0.5, 0.5, 0.0);
        transformed
    });
    (ladder_vertices, texcoords)
}

fn gen_fence_rail(
    cube: &[Vert],
    tc: &[Tc],
    normals: &[Norm],
    x: f32,
    z: f32,
    sx: f32,
    sz: f32,
) -> BlockMesh {
    let mut fence_rail = vec![];
    let top = transform_vertices(cube, |v| {
        let mut transformed = v;
        transformed.x *= sx;
        transformed.y *= 1.0 / 8.0;
        transformed.z *= sz;
        transformed += Vert4::new(0.5 + x, 0.5 + 5.0 / 16.0, 0.5 + z, 0.0);
        transformed
    });
    let bot = transform_vertices(cube, |v| {
        let mut transformed = v;
        transformed.x *= sx;
        transformed.y *= 1.0 / 8.0;
        transformed.z *= sz;
        transformed += Vert4::new(0.5 + x, 0.5 - 2.0 / 16.0, 0.5 + z, 0.0);
        transformed
    });
    fence_rail.extend(top);
    fence_rail.extend(bot);

    let mut fence_tc = vec![];
    let tc = transform_tc(tc, |v, i| {
        let mut tc = v;
        if normals[i].y != 0.0 && sz > sx {
            tc.x *= 1.0 / 4.0;
            tc.x += 6.0 / 16.0;
            tc.y *= sx.max(sz);
            std::mem::swap(&mut tc.x, &mut tc.y);
        } else {
            tc.x *= sx.max(sz);
            tc.y *= 1.0 / 4.0;
            tc.y += 6.0 / 16.0;
        }
        tc
    });
    fence_tc.extend(tc.clone());
    fence_tc.extend(tc);

    (fence_rail, fence_tc)
}

fn gen_fence_vertices(block: Block) -> BlockMesh {
    let vertices = generate_mesh_vertices(&CUBE, &CUBE_INDICES);
    let normals = generate_mesh_normals(&vertices);
    let texcoords = generate_mesh_texcoords(&TEX_COORDS, &CUBE_TEX_INDICES);
    let mut fence_vertices = transform_vertices(&vertices, |v| {
        let mut transformed = v;
        transformed.x *= 1.0 / 4.0;
        transformed.z *= 1.0 / 4.0;
        transformed += Vert4::new(0.5, 0.5, 0.5, 0.0);
        transformed
    });
    let mut fence_texcoords = transform_tc(&texcoords, |v, i| {
        let mut tc = v;
        if normals[i].y != 0.0 {
            tc *= 2.0 / 16.0;
            tc.y += 7.0 / 16.0;
            tc.x += 6.0 / 16.0;
            return tc;
        }
        tc.x *= 4.0 / 16.0;
        tc.x += 6.0 / 16.0;
        tc
    });

    let (verts, tc) = if block.geometry & 1 << 0 != 0 && block.geometry & 1 << 2 != 0 {
        gen_fence_rail(&vertices, &texcoords, &normals, 0.0, 0.0, 1.0, 1.0 / 8.0)
    } else if block.geometry & 1 << 0 != 0 {
        gen_fence_rail(&vertices, &texcoords, &normals, 0.25, 0.0, 0.5, 1.0 / 8.0)
    } else if block.geometry & 1 << 2 != 0 {
        gen_fence_rail(&vertices, &texcoords, &normals, -0.25, 0.0, 0.5, 1.0 / 8.0)
    } else {
        (vec![], vec![])
    };
    fence_vertices.extend(verts);
    fence_texcoords.extend(tc);

    let (verts, tc) = if block.geometry & 1 << 1 != 0 && block.geometry & 1 << 3 != 0 {
        gen_fence_rail(&vertices, &texcoords, &normals, 0.0, 0.0, 1.0 / 8.0, 1.0)
    } else if block.geometry & 1 << 1 != 0 {
        gen_fence_rail(&vertices, &texcoords, &normals, 0.0, 0.25, 1.0 / 8.0, 0.5)
    } else if block.geometry & 1 << 3 != 0 {
        gen_fence_rail(&vertices, &texcoords, &normals, 0.0, -0.25, 1.0 / 8.0, 0.5)
    } else {
        (vec![], vec![])
    };
    fence_vertices.extend(verts);
    fence_texcoords.extend(tc);

    (fence_vertices, fence_texcoords)
}

fn gen_seed_vertices(_block: Block) -> BlockMesh {
    let vertices = generate_mesh_vertices(&CUBE, &QUAD_INDICES);
    let texcoords = generate_mesh_texcoords(&TEX_COORDS, &QUAD_INDICES);
    let ladder_vertices = transform_vertices(&vertices, |v| {
        let mut transformed = v;
        transformed = Matrix4::from_angle_x(Deg(90.0)) * transformed;
        transformed = Matrix4::from_angle_x(Deg(90.0)) * transformed;
        transformed.y -= 15.0 / 16.0;
        transformed += Vert4::new(0.5, 0.5, 0.5, 0.0);
        transformed
    });
    (ladder_vertices, texcoords)
}

fn gen_gate_door(cube: &[Vert], tc: &[Tc], normals: &[Norm]) -> BlockMesh {
    let mut door_verts = vec![];
    let top = transform_vertices(cube, |v| {
        let mut transformed = v;
        transformed.y *= 1.0 / 8.0;
        transformed.z *= 1.0 / 8.0;
        transformed.y += 5.0 / 16.0;
        transformed
    });
    let bot = transform_vertices(cube, |v| {
        let mut transformed = v;
        let y = transformed.y;
        transformed.y *= 1.0 / 8.0;
        transformed.z *= 1.0 / 8.0;
        transformed.y -= 2.0 / 16.0;
        if transformed.x > 0.0 {
            if y > 0.0 {
                transformed.y += 5.0 / 16.0;
                transformed.x -= 4.0 / 16.0;
            } else {
                transformed.y += 7.0 / 16.0;
            }
        } else {
            transformed.y -= 1.0 / 16.0;
        }
        transformed
    });
    door_verts.extend(top);
    door_verts.extend(bot);

    let mut door_tc = vec![];
    let tc = transform_tc(tc, |v, i| {
        let mut tc = v;
        if normals[i].x != 0.0 {
            tc.x *= 1.0 / 4.0;
            tc.y *= 1.0 / 4.0;
        } else {
            tc.y *= 1.0 / 4.0;
        }
        tc.y += 6.0 / 16.0;
        tc
    });
    door_tc.extend(tc.clone());
    door_tc.extend(tc.clone());

    (door_verts, door_tc)
}

fn gen_gate_vertices(block: Block) -> BlockMesh {
    let vertices = generate_mesh_vertices(&CUBE, &CUBE_INDICES);
    let normals = generate_mesh_normals(&vertices);
    let texcoords = generate_mesh_texcoords(&TEX_COORDS, &CUBE_TEX_INDICES);
    let post = transform_vertices(&vertices, |v| {
        let mut transformed = v;
        transformed.x *= 1.0 / 4.0;
        transformed.z *= 1.0 / 4.0;
        transformed += Vert4::new(0.5, 0.5, 0.5, 0.0);
        transformed
    });
    let tc = transform_tc(&texcoords, |v, i| {
        let mut tc = v;
        if normals[i].y != 0.0 {
            tc *= 2.0 / 16.0;
            tc.y += 7.0 / 16.0;
            tc.x += 6.0 / 16.0;
            return tc;
        }
        tc.x *= 4.0 / 16.0;
        tc.x += 6.0 / 16.0;
        tc
    });
    let mut gate_vertices = vec![];
    let mut gate_texcoords = vec![];
    gate_vertices.extend(transform_vertices(&post, |v| {
        let mut transformed = v;
        match block.orientation() {
            1 | 4 => transformed.z -= 8.0 / 16.0,
            2 | 5 => transformed.x -= 8.0 / 16.0,
            _ => {}
        }
        transformed
    }));
    gate_texcoords.extend(tc.clone());
    gate_vertices.extend(transform_vertices(&post, |v| {
        let mut transformed = v;
        match block.orientation() {
            1 | 4 => transformed.z += 8.0 / 16.0,
            2 | 5 => transformed.x += 8.0 / 16.0,
            _ => {}
        }
        transformed
    }));
    gate_texcoords.extend(tc);

    let (door_verts, door_tc) = gen_gate_door(&vertices, &texcoords, &normals);
    let door_verts_trans = transform_vertices(&door_verts, |v| {
        let mut transformed = v;
        if block.reflection() == 1 {
            transformed = Matrix4::from_angle_y(Deg(90.0)) * transformed;
            transformed.z -= 0.5;
            transformed.x -= 0.5;
        }
        match block.orientation() {
            1 => transformed = Matrix4::from_angle_y(Deg(270.0)) * transformed,
            2 => transformed = Matrix4::from_angle_y(Deg(180.0)) * transformed,
            4 => transformed = Matrix4::from_angle_y(Deg(90.0)) * transformed,
            _ => {}
        }
        transformed += Vert4::new(0.5, 0.5, 0.5, 0.0);
        transformed
    });
    gate_vertices.extend(door_verts_trans);
    gate_texcoords.extend(door_tc);

    (gate_vertices, gate_texcoords)
}

fn gen_door_vertices(block: Block) -> BlockMesh {
    let vertices = generate_mesh_vertices(&CUBE, &CUBE_INDICES);
    let normals = generate_mesh_normals(&vertices);
    let texcoords = generate_mesh_texcoords(&TEX_COORDS, &CUBE_TEX_INDICES);
    let door = transform_vertices(&vertices, |v| {
        let mut transformed = v;
        transformed.x *= 1.0 / 8.0;
        transformed -= Vert4::new(7.0 / 16.0, 0.0, 0.0, 0.0);
        if block.reflection() == 1 {
            transformed = Matrix4::from_angle_y(Deg(-90.0)) * transformed;
        }
        match block.orientation() {
            2 => {
                std::mem::swap(&mut transformed.x, &mut transformed.z);
                transformed.x *= -1.0;
            }
            4 => {
                transformed.z *= -1.0;
                transformed.x *= -1.0;
            }
            5 => {
                std::mem::swap(&mut transformed.x, &mut transformed.z);
                transformed.z *= -1.0;
            }
            _ => {}
        }
        transformed += Vert4::new(0.5, 0.5, 0.5, 0.0);
        transformed
    });
    let tc = transform_tc(&texcoords, |v, i| {
        let mut tc = v;
        if normals[i].z != 0.0 || normals[i].y != 0.0 {
            tc.x *= 1.0 / 8.0;
            tc.x += 7.0 / 16.0;
        }
        if block.reflection() == 1 && normals[i].x != 0.0 {
            tc.x = 1.0 - tc.x;
        }
        tc
    });
    (door, tc)
}

pub fn add_nonvoxel_vertices(
    chunk: &Chunk,
    xyz: Int3,
    vert_data: &mut ChunkData,
    cached_meshes: &mut HashMap<(u8, u8), BlockMesh>,
) {
    let (x, y, z) = xyz;
    let block = chunk.get_block_relative(x as usize, y as usize, z as usize);

    if !block.non_voxel_geometry() {
        return;
    }

    let id = match block.id {
        //Fence and gate
        76 | 78 => 6,
        //Door
        79 => 80,
        _ => block.id,
    };

    let light = chunk.get_light_relative(x as usize, y as usize, z as usize);
    let key = (block.id, block.geometry);
    if let Some((vert, tc)) = cached_meshes.get(&key) {
        add_mesh_to_chunk(xyz, id, vert, tc, vert_data, light);
    }

    let (vert, tc) = match block.id {
        //Torch
        71..=74 => gen_torch_vertices(block),
        //Ladder
        75 => gen_ladder_vertices(block),
        //Fence
        76 => gen_fence_vertices(block),
        //Seed
        77 => gen_seed_vertices(block),
        //Gate
        78 => gen_gate_vertices(block),
        //Bottom door
        79 | 81 => gen_door_vertices(block),
        //Top door
        _ => (vec![], vec![]),
    };

    add_mesh_to_chunk(xyz, id, &vert, &tc, vert_data, light);
    cached_meshes.insert(key, (vert, tc));
}
