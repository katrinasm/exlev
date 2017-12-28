use std::ops;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SnesColor {
    red: f32,
    green: f32,
    blue: f32,
}

impl SnesColor {
    pub fn from_rgb24(red: u8, green: u8, blue: u8) -> SnesColor {
        SnesColor {
            red: (red as f32) / 255.0,
            green: (green as f32) / 255.0,
            blue: (blue as f32) / 255.0,
        }
    }
    
    pub fn from_rgb15(red: u8, green: u8, blue: u8) -> SnesColor {
        assert!(red < 32 && blue < 32 && green < 32, "color component out of range");
        SnesColor {
            red: (red as f32) / 31.0,
            green: (green as f32) / 31.0,
            blue: (blue as f32) / 31.0,
        }
    }
    
    pub fn from_snes(word: u16) -> SnesColor {
        let r = (word as u8) & 0x1f;
        let g = ((word >> 5) as u8) & 0x1f;
        let b = ((word >> 10) as u8) & 0x1f;
        SnesColor::from_rgb15(r, g, b)
    }

    pub fn from_fs32(red: f32, green: f32, blue: f32) -> SnesColor {
        assert!(
            red >= 0.0 && blue >= 0.0 && green >= 0.0
            && red <= 1.0 && blue <= 1.0 && green <= 1.0,
            "color component out of range"
        );
        SnesColor { red, green, blue }
    }
    
    pub fn to_rgb24(&self) -> (u8, u8, u8) {
        (
            (kir(self.red) * 255.0) as u8,
            (kir(self.green) * 255.0) as u8,
            (kir(self.blue) * 255.0) as u8,
        )
    }
    
    pub fn to_rgb15(&self) -> (u8, u8, u8) {
        (
            (kir(self.red) * 31.0) as u8,
            (kir(self.green) * 31.0) as u8,
            (kir(self.blue) * 31.0) as u8,
        )
    }
    
    pub fn to_snes(&self) -> u16 {
        ((kir(self.red) * 31.0) as u16)
        | (((kir(self.green) * 31.0) as u16) << 5)
        | (((kir(self.blue) * 31.0) as u16) << 10)
    }
    
    pub fn to_fs32(&self) -> (f32, f32, f32) {
        (kir(self.red), kir(self.green), kir(self.blue))
    }
    
    pub fn half(&self) -> SnesColor {
        SnesColor {
            red: self.red / 2.0,
            green: self.green / 2.0,
            blue: self.blue / 2.0,
        }
    }
}

impl ops::Add<SnesColor> for SnesColor {
    type Output = SnesColor;
    fn add(self, other: SnesColor) -> SnesColor {
        SnesColor {
            red: self.red + other.red,
            green: self.green + other.green,
            blue: self.blue + other.blue,
        }
    }
}

impl ops::AddAssign<SnesColor> for SnesColor {
    fn add_assign(&mut self, other: SnesColor) {
        self.red += other.red;
        self.green += other.green;
        self.blue += other.blue;
    }
}

impl ops::Sub<SnesColor> for SnesColor {
    type Output = SnesColor;
    fn sub(self, other: SnesColor) -> SnesColor {
        SnesColor {
            red: self.red - other.red,
            green: self.green - other.green,
            blue: self.blue - other.blue,
        }
    }
}

impl ops::SubAssign<SnesColor> for SnesColor {
    fn sub_assign(&mut self, other: SnesColor) {
        self.red -= other.red;
        self.green -= other.green;
        self.blue -= other.blue;
    }
}

// for "keep-in-range"
// sometimes operating on a color might produce a value > 1.0 or < 0.0
// these can be reasonably rounded back down to 1 or up to 0 when deconverting
fn kir(v: f32) -> f32 {
    v.min(1.0).max(0.0)
}

pub struct SnesPal {
    pub bg: SnesColor,
    pub colors: [SnesColor; 256],
}

impl SnesPal {
    pub fn from_lm_pal(pal_bytes: &[u8]) -> Option<SnesPal> {
        if pal_bytes.len() < 768 {
            return None;
        }
        
        let black = SnesColor::from_fs32(0.0, 0.0, 0.0);
        let mut colors = [black; 256];
        
        let bg = SnesColor::from_rgb24(pal_bytes[0], pal_bytes[1], pal_bytes[2]);
        
        
        for (i, chunk) in pal_bytes[.. 768].chunks(3).enumerate() {
            if i & 0xf != 0 { // skip the first color of each row
                colors[i] = SnesColor::from_rgb24(chunk[0], chunk[1], chunk[2]);
            }
        }
        
        Some(SnesPal {bg, colors})
    }
    
    pub fn to_binary_snes(&self) -> Vec<u8> {
        let mut v = Vec::with_capacity(512);
        for color in self.iter() {
            let w = color.to_snes();
            v.push(w as u8);
            v.push((w >> 8) as u8);
        }
        v
    }
    
    pub fn iter(&self) -> SnesPalIter {
        SnesPalIter { idx: 0, pal: &self }
    }
}

impl Clone for SnesPal {
    fn clone(&self) -> SnesPal {
        let mut new = SnesPal { bg: self.bg, colors: [self.bg; 256] };
        for (d, s) in new.colors.iter_mut().zip(self.colors.iter()) {
            *d = *s;
        };
        new
    }
}

impl ::std::fmt::Debug for SnesPal {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
        write!(f, "SnesPal {{ bg: ({}, {}, {}), colors: [",
            self.bg.red, self.bg.green, self.bg.blue,
        )?;
        for i in 0 .. self.colors.len() {
            let color = self.colors[i];
            write!(f, "({}, {}, {}), ", color.red, color.green, color.blue)?;
        }
        write!(f, "] }}")?;
        Ok(())
    }
}

pub struct SnesPalIter<'p> {
    pal: &'p SnesPal,
    idx: usize,
}

impl<'p> Iterator for SnesPalIter<'p> {
    type Item = SnesColor;
    fn next(&mut self) -> Option<SnesColor> {
        if self.idx == 0 {
            self.idx += 1;
            Some(self.pal.bg)
        } else if self.idx <= 256 {
            let i = self.idx - 1;
            self.idx += 1;
            Some(self.pal.colors[i])
        } else {
            None
        }
    }
}
