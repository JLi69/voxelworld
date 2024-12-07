use std::fs::File;
use std::os::raw::c_void;

pub fn load_image_pixels(path: &str) -> Result<(Vec<u32>, png::OutputInfo), String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    let decoder = png::Decoder::new(file);
    let mut reader = decoder.read_info().map_err(|e| e.to_string())?;

    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).map_err(|e| e.to_string())?;

    let mut pixels = vec![0; buf.len() / 4];
    for i in 0..(buf.len() / 4) {
        if i * 4 + 3 >= buf.len() {
            break;
        }

        let (r, g, b, a) = (
            buf[i * 4] as u32,
            buf[i * 4 + 1] as u32,
            buf[i * 4 + 2] as u32,
            buf[i * 4 + 3] as u32,
        );

        pixels[i] = a << 24 | b << 16 | g << 8 | r;
    }

    Ok((pixels, info))
}

pub struct Texture {
    id: u32,
}

impl Texture {
    //Creates a blank texture with id 0
    pub fn new() -> Self {
        Self { id: 0 }
    }

    //Attempt to load texture from a PNG file
    //(assume that the PNG file has its colors encoded in RGBA)
    pub fn load_from_file(path: &str) -> Result<Self, String> {
        let file_res = File::open(path);

        match file_res {
            Ok(file) => {
                //Attempt to read from the texture file
                let decoder = png::Decoder::new(file);
                let mut reader = decoder.read_info().map_err(|e| e.to_string())?;
                let mut buf = vec![0u8; reader.output_buffer_size()];
                let info = reader.next_frame(&mut buf).map_err(|e| e.to_string())?;
                let bytes = &buf[..info.buffer_size()];

                let mut texture = 0;
                unsafe {
                    gl::GenTextures(1, &mut texture);
                    gl::BindTexture(gl::TEXTURE_2D, texture);
                    gl::TextureParameteri(texture, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
                    gl::TextureParameteri(texture, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
                    gl::TexImage2D(
                        gl::TEXTURE_2D,
                        0,
                        gl::RGBA as i32,
                        info.width as i32,
                        info.height as i32,
                        0,
                        gl::RGBA,
                        gl::UNSIGNED_BYTE,
                        bytes.as_ptr() as *const c_void,
                    );
                    gl::GenerateMipmap(gl::TEXTURE_2D);
                }

                Ok(Self { id: texture })
            }
            Err(msg) => {
                eprintln!("Failed to open file: {path}");
                eprintln!("{msg}");
                Err(msg.to_string())
            }
        }
    }

    pub fn gen_texture(&mut self) {
        unsafe {
            gl::GenTextures(1, &mut self.id);
        }
    }

    pub fn get_id(&self) -> u32 {
        self.id
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.id);
        }
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.id);
        }
    }
}
