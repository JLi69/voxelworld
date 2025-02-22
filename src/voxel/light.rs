#[derive(Clone, Copy, Debug)]
pub struct Light {
    //[4 bits sky][4 bits red][4 bits green][4 bits blue]
    light_data: u16,
}

#[derive(Clone, Copy)]
pub struct LightUpdate {
    //If any of these values are None, then that means no update
    pub sky: Option<u16>,
    pub r: Option<u16>,
    pub g: Option<u16>,
    pub b: Option<u16>,
}

#[derive(Clone, Copy)]
pub struct LightSrc {
    pub r: u16,
    pub g: u16,
    pub b: u16,
}

impl LightSrc {
    pub fn new(rval: u16, gval: u16, bval: u16) -> Self {
        Self {
            r: rval,
            g: gval,
            b: bval,
        }
    }
}

pub type LU = LightUpdate;

impl LightUpdate {
    pub fn new(
        skyupdate: Option<u16>,
        rupdate: Option<u16>,
        gupdate: Option<u16>,
        bupdate: Option<u16>,
    ) -> Self {
        Self {
            sky: skyupdate,
            r: rupdate,
            g: gupdate,
            b: bupdate,
        }
    }

    pub fn from_src(src: LightSrc) -> Self {
        Self::new(None, Some(src.r), Some(src.g), Some(src.b))
    }
}

macro_rules! get_channel {
    ($channel:ident, $bits:expr) => {
        pub fn $channel<T: From<u16>>(&self) -> T {
            ((self.light_data >> $bits) & 0xf).into()
        }
    };
}

macro_rules! set_channel {
    ($channel:ident, $bits:expr) => {
        pub fn $channel<T: Into<u16>>(&mut self, v: T) {
            self.light_data &= !(0xf << $bits);
            self.light_data |= (v.into() << $bits);
        }
    };
}

macro_rules! get_client {
    ($fn_name: ident, $channel:ident) => {
        pub fn $fn_name<T: From<u16> + Ord>(&self) -> T {
            self.sky::<T>().max(self.$channel())
        }
    };
}

impl Light {
    //All channels are set to 0 (black)
    pub fn black() -> Self {
        Self { light_data: 0 }
    }

    //All channels are set to the corresponding values
    pub fn new<T: Into<u16>>(sky: T, r: T, g: T, b: T) -> Self {
        Self {
            light_data: (sky.into() << 12) | (r.into() << 8) | (g.into() << 4) | b.into(),
        }
    }

    //Get the channel values
    get_channel!(blue, 0);
    get_channel!(green, 4);
    get_channel!(red, 8);
    get_channel!(sky, 12);

    //Set the channel values
    set_channel!(set_blue, 0);
    set_channel!(set_green, 4);
    set_channel!(set_red, 8);
    set_channel!(set_sky, 12);

    //Returns the client values
    //(returns the max of sky light value and the channel)
    get_client!(client_blue, blue);
    get_client!(client_green, green);
    get_client!(client_red, red);

    //Returns the rgb values as a tuple (without the sky)
    pub fn get_rgb<T: From<u16>>(&self) -> (T, T, T) {
        (self.red(), self.green(), self.blue())
    }

    //Returns all the client values in a tuple (red, green, blue)
    pub fn get_client<T: From<u16> + Ord>(&self) -> (T, T, T) {
        (self.client_red(), self.client_green(), self.client_blue())
    }

    //rgb channels
    pub fn r(&self) -> u16 {
        self.red()
    }

    pub fn g(&self) -> u16 {
        self.green()
    }

    pub fn b(&self) -> u16 {
        self.blue()
    }

    //Sky
    pub fn skylight(&self) -> u16 {
        self.sky()
    }

    //Update channels
    pub fn update(&mut self, light_update: LU) {
        if let Some(sky) = light_update.sky {
            self.set_sky(sky);
        }

        if let Some(r) = light_update.r {
            self.set_red(r);
        }

        if let Some(g) = light_update.g {
            self.set_green(g);
        }

        if let Some(b) = light_update.b {
            self.set_blue(b);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_set_red() {
        let mut light = Light::black();
        for v in (0..16u16).rev() {
            light.set_red(v);
            assert_eq!(light.r(), v);
            assert_eq!(light.g(), 0);
            assert_eq!(light.b(), 0);
            assert_eq!(light.skylight(), 0);
        }
    }

    #[test]
    pub fn test_set_green() {
        let mut light = Light::black();
        for v in (0..16u16).rev() {
            light.set_green(v);
            assert_eq!(light.r(), 0);
            assert_eq!(light.g(), v);
            assert_eq!(light.b(), 0);
            assert_eq!(light.skylight(), 0);
        }
    }

    #[test]
    pub fn test_set_blue() {
        let mut light = Light::black();
        for v in (0..16u16).rev() {
            light.set_blue(v);
            assert_eq!(light.r(), 0);
            assert_eq!(light.g(), 0);
            assert_eq!(light.b(), v);
            assert_eq!(light.skylight(), 0);
        }
    }

    #[test]
    pub fn test_set_sky() {
        let mut light = Light::black();
        for v in (0..16u16).rev() {
            light.set_sky(v);
            assert_eq!(light.r(), 0);
            assert_eq!(light.g(), 0);
            assert_eq!(light.b(), 0);
            assert_eq!(light.skylight(), v);
        }
    }

    #[test]
    pub fn test_light_creation() {
        let light = Light::new(4u16, 1u16, 2u16, 3u16);
        assert_eq!(light.skylight(), 4);
        assert_eq!(light.r(), 1);
        assert_eq!(light.g(), 2);
        assert_eq!(light.b(), 3);
        assert_eq!(light.get_rgb(), (1, 2, 3));
        assert_eq!(light.get_client(), (4, 4, 4));
    }

    #[test]
    pub fn test_set() {
        let mut light = Light::black();
        assert_eq!(light.skylight(), 0);
        assert_eq!(light.r(), 0);
        assert_eq!(light.g(), 0);
        assert_eq!(light.b(), 0);

        light.set_sky(8u16);
        light.set_red(2u16);
        light.set_green(8u16);
        light.set_blue(14u16);
        assert_eq!(light.get_rgb(), (2, 8, 14));
        assert_eq!(light.get_client(), (8, 8, 14));
    }
}
