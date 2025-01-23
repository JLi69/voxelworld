/*
 * This is a file of simple generated models
 * */

use std::ffi::c_void;
use std::mem::size_of;

//Cube vertex data
#[rustfmt::skip]
pub const CUBE: [f32; 24] = [
    -0.5, -0.5, -0.5,
    0.5, -0.5, -0.5,
    -0.5, -0.5, 0.5,
    0.5, -0.5, 0.5,

    -0.5, 0.5, -0.5, 
    0.5, 0.5, -0.5,
    -0.5, 0.5, 0.5,
    0.5, 0.5, 0.5,
];

#[rustfmt::skip]
pub const TEX_COORDS: [f32; 8] = [
    0.0, 0.0,
    1.0, 0.0,
    0.0, 1.0,
    1.0, 1.0,
];

#[rustfmt::skip]
//Texture coordinates for a cube
pub const CUBE_TEX_INDICES: [u32; 36] = [
    0, 1, 2,
    3, 2, 1,

    0, 2, 3,
    3, 1, 0,

    0, 1, 2,
    3, 2, 1,

    2, 0, 1,
    1, 3, 2,

    2, 3, 1,
    1, 0, 2,

    2, 1, 0,
    1, 2, 3,
];

//This is for a cube
#[rustfmt::skip]
pub const CUBE_INDICES: [u32; 36] = [
    0, 1, 2,
    3, 2, 1,

    0, 4, 5,
    5, 1, 0,

    0, 2, 4,
    6, 4, 2,

    6, 2, 3,
    3, 7, 6,

    5, 7, 3,
    3, 1, 5,

    6, 5, 4,
    5, 6, 7,
];

//quad texture coordinates
pub const QUAD_INDICES: [u32; 6] = [
    0, 1, 2,
    3, 2, 1,
];

//2D quad vertex data
#[rustfmt::skip]
pub const QUAD_2D: [f32; 8] = [
    1.0, 1.0,
    1.0, -1.0,
    -1.0, -1.0,
    -1.0, 1.0,
];

#[rustfmt::skip]
pub const QUAD_2D_INDICES: [u32; 6] = [
    0, 1, 2,
    3, 0, 2,
];

#[derive(Clone)]
pub struct Vao {
    vao_id: u32,
    buffers: Vec<u32>,
    vert_count: i32,
}

impl Vao {
    //Create new empty vao
    pub fn new(buffer_count: usize) -> Self {
        Self {
            vao_id: 0,
            buffers: vec![0; buffer_count],
            vert_count: 0,
        }
    }

    //Bind vertex array
    pub fn bind(&self) {
        unsafe {
            gl::BindVertexArray(self.vao_id);
        }
    }

    //Draw the vao with triangles as the primitive
    pub fn draw_elements(&self) {
        unsafe {
            gl::DrawElements(
                gl::TRIANGLES,
                self.vert_count,
                gl::UNSIGNED_INT,
                std::ptr::null(),
            );
        }
    }
}

//Create a cube
pub fn gen_cube_vao() -> Vao {
    let mut cube_vao = Vao::new(2);

    unsafe {
        //Generate buffers
        gl::GenVertexArrays(1, &mut cube_vao.vao_id);
        gl::GenBuffers(2, &mut cube_vao.buffers[0]);

        //Bind vao
        gl::BindVertexArray(cube_vao.vao_id);
        //Vertex data
        gl::BindBuffer(gl::ARRAY_BUFFER, cube_vao.buffers[0]);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (CUBE.len() * size_of::<f32>()) as isize,
            &CUBE[0] as *const f32 as *const c_void,
            gl::STATIC_DRAW,
        );
        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            size_of::<f32>() as i32 * 3,
            std::ptr::null::<f32>() as *const c_void,
        );
        gl::EnableVertexAttribArray(0);
        //Indices
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, cube_vao.buffers[1]);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            (CUBE_INDICES.len() * size_of::<f32>()) as isize,
            &CUBE_INDICES[0] as *const u32 as *const c_void,
            gl::STATIC_DRAW,
        );
    }

    cube_vao.vert_count = CUBE_INDICES.len() as i32;

    cube_vao
}

pub fn gen_quad2d_vao() -> Vao {
    let mut quad_vao = Vao::new(2);

    unsafe {
        //Generate buffers
        gl::GenVertexArrays(1, &mut quad_vao.vao_id);
        gl::GenBuffers(2, &mut quad_vao.buffers[0]);

        //Bind vao
        gl::BindVertexArray(quad_vao.vao_id);
        //Vertex data
        gl::BindBuffer(gl::ARRAY_BUFFER, quad_vao.buffers[0]);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (QUAD_2D.len() * size_of::<f32>()) as isize,
            &QUAD_2D[0] as *const f32 as *const c_void,
            gl::STATIC_DRAW,
        );
        gl::VertexAttribPointer(
            0,
            2,
            gl::FLOAT,
            gl::FALSE,
            size_of::<f32>() as i32 * 2,
            std::ptr::null::<f32>() as *const c_void,
        );
        gl::EnableVertexAttribArray(0);
        //Indices
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, quad_vao.buffers[1]);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            (QUAD_2D_INDICES.len() * size_of::<f32>()) as isize,
            &QUAD_2D_INDICES[0] as *const u32 as *const c_void,
            gl::STATIC_DRAW,
        );
    }

    quad_vao.vert_count = QUAD_2D_INDICES.len() as i32;

    quad_vao
}
