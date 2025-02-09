use crate::assets::Texture;
use crate::game::Game;
use crate::gfx;

pub fn display_hud(gamestate: &Game, w: i32, h: i32) {
    let stuck = gamestate.player.get_head_stuck_block(&gamestate.world);
    //Display the current block the player has selected
    if gamestate.display_hud && stuck.is_none() {
        //Display selection outline
        gfx::display::display_selected_outline(gamestate);
    }

    //Display suffocation screen
    if gamestate.player.suffocating(&gamestate.world) {
        gfx::display::display_suffocation_screen(gamestate, w, h);
    }

    //Clear depth buffer
    unsafe {
        gl::Clear(gl::DEPTH_BUFFER_BIT);
    }
    if gamestate.display_hud {
        //Display crosshair
        gfx::display::display_crosshair(gamestate, w, h);
        //Display held item
        gfx::display::display_hand_item(gamestate);
        //Display hotbar
        gfx::display::display_hotbar(gamestate, w, h);
    }
}

//Returns (framebuffer, depth buffer object, water texture)
pub fn setup_water_framebuff() -> (u32, u32, Texture) {
    //water framebuffer
    let mut water_framebuffer = 0u32;
    let mut water_frame_color = Texture::new();
    let mut depth_rbo = 0u32;
    water_frame_color.gen_texture();
    unsafe {
        //Create render buffer
        gl::GenRenderbuffers(1, &mut depth_rbo);
        gl::BindRenderbuffer(gl::RENDERBUFFER, depth_rbo);
        gl::RenderbufferStorage(gl::RENDERBUFFER, gl::DEPTH24_STENCIL8, 960, 640);

        //Initialize frame buffer
        gl::GenFramebuffers(1, &mut water_framebuffer);
        gl::BindFramebuffer(gl::FRAMEBUFFER, water_framebuffer);
        water_frame_color.bind();
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGBA as i32,
            960,
            640,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            std::ptr::null(),
        );
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::FramebufferTexture2D(
            gl::FRAMEBUFFER,
            gl::COLOR_ATTACHMENT0,
            gl::TEXTURE_2D,
            water_frame_color.get_id(),
            0,
        );
        gl::FramebufferRenderbuffer(
            gl::FRAMEBUFFER,
            gl::DEPTH_STENCIL_ATTACHMENT,
            gl::RENDERBUFFER,
            depth_rbo,
        );

        //Check framebuffer status
        if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
            eprintln!("ERROR: framebuffer is not complete!");
        }

        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
    }

    (water_framebuffer, depth_rbo, water_frame_color)
}

unsafe fn update_depth_rbo(depth_rbo: u32, w: i32, h: i32) {
    gl::BindRenderbuffer(gl::RENDERBUFFER, depth_rbo);

    let mut rbo_width = 0;
    let mut rbo_height = 0;
    gl::GetRenderbufferParameteriv(gl::RENDERBUFFER, gl::RENDERBUFFER_WIDTH, &mut rbo_width);
    gl::GetRenderbufferParameteriv(gl::RENDERBUFFER, gl::RENDERBUFFER_HEIGHT, &mut rbo_height);

    if rbo_width == w && rbo_height == h {
        gl::BindRenderbuffer(gl::RENDERBUFFER, 0); //Unbind render buffer
        return;
    }

    gl::RenderbufferStorage(gl::RENDERBUFFER, gl::DEPTH24_STENCIL8, w, h);
    gl::BindRenderbuffer(gl::RENDERBUFFER, 0); //Unbind render buffer
}

unsafe fn update_water_texture(water_frame_color: &Texture, w: i32, h: i32) {
    water_frame_color.bind();
    let mut water_frame_w = 0;
    let mut water_frame_h = 0;
    gl::GetTexLevelParameteriv(gl::TEXTURE_2D, 0, gl::TEXTURE_WIDTH, &mut water_frame_w);
    gl::GetTexLevelParameteriv(gl::TEXTURE_2D, 0, gl::TEXTURE_HEIGHT, &mut water_frame_h);

    if water_frame_w == w && water_frame_h == h {
        gl::BindTexture(gl::TEXTURE_2D, 0); //Unbind texture
        return;
    }

    gl::TexImage2D(
        gl::TEXTURE_2D,
        0,
        gl::RGBA as i32,
        w,
        h,
        0,
        gl::RGBA,
        gl::UNSIGNED_BYTE,
        std::ptr::null(),
    );
    gl::BindTexture(gl::TEXTURE_2D, 0); //Unbind texture
}

pub fn update_water_frame_dimensions(depth_rbo: u32, water_frame_color: &Texture, w: i32, h: i32) {
    unsafe {
        update_depth_rbo(depth_rbo, w, h);
        update_water_texture(water_frame_color, w, h);
    }
}
