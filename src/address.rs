#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Address { pc: usize, map: Mapper }

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Mapper {Lorom, Hirom, Exlorom, Exhirom, Sfxrom, Sa1rom([u8; 4]), Sddrom([u8; 4])}

pub const BOOT_SA1ROM: Mapper = Mapper::Sa1rom([0, 1, 2, 3]);
pub const BOOT_SDDROM: Mapper = Mapper::Sddrom([0, 1, 2, 3]);

impl Mapper {
    pub fn region_base(&self, n: u8) -> u32 {
        assert!(n < 4, "called region_base with a too-high region number");
        match *self {
            Sa1rom(_) | Sddrom(_) => 0xc0_0000 + n as u32 * 0x10_0000,
            _ => panic!("called region_base on a mapper without regions"),
        }
    }
}

use address::Mapper::*;

impl Address {
    pub fn new_from_pc(pc: usize, map: Mapper) -> Option<Address> {
        let in_range = pc < match map {
            Lorom => 0x40_0000,
            Hirom => 0x40_0000,
            Exlorom => 0x80_0000,
            Exhirom => 0x80_0000,
            Sfxrom => 0x20_0000,
            Sa1rom(_) => 0x80_0000,
            Sddrom(_) => 0x80_0000,
        };
        if in_range {
            Some(Address {pc, map})
        } else {
            None
        }
    }

    pub fn new_from_snes(ofs: usize, map: Mapper) -> Option<Address> {
        if ofs >= 0x7e_0000 && ofs < 0x80_0000 {
            return None;
        };
        let in_range = match map {
            Lorom | Exlorom => ofs & 0x00_8000 != 0 && !(ofs >= 0x70_0000 && ofs < 0x80_0000),
            Hirom | Exhirom => ofs & 0x40_0000 != 0 || ofs & 0x00_8000 != 0,
            Sfxrom => unimplemented!(),
            Sa1rom(_) => unimplemented!(),
                //ofs >= 0xc0_0000 || ofs & 0x00_8000 != 0 && !(ofs >= 0x40_0000 && ofs < 0x80_0000),
            Sddrom(_) => unimplemented!(),
        };
        if !in_range {
            return None;
        };
        Some(
            Address {
                map,
                pc: match map {
                    Lorom => (ofs & 0x7f_0000) >> 1 | ofs & 0x00_7fff,
                    Exlorom => (ofs & 0xff_0000) >> 1 | ofs & 0x00_7fff,
                    Hirom => ofs & 0x3f_ffff,
                    Exhirom => (ofs & 0x80_0000) >> 1 | ofs & 0x3f_ffff,
                    _ => unimplemented!(),
                }
            }
        )
    }
    
    pub fn new_from_snes_bytes(bytes: &[u8], mapper: Mapper) -> Option<Address> {
        assert!(bytes.len() >= 3,
            "called Address::new_from_snes_bytes with less than a pointer's worth of bytes");
        let (lo, hi, bk) = (bytes[0] as usize, bytes[1] as usize, bytes[2] as usize);
        Address::new_from_snes(lo | (hi << 8) | (bk << 16), mapper)
    }

    pub fn pc_ofs(&self) -> usize {
        self.pc
    }

    pub fn snes_ofs(&self) -> Option<u32> {
        let pc32 = self.pc as u32;
        match self.map {
            Lorom => Some((pc32 & 0x3f_8000) << 1 | pc32 & 0x00_7fff | 0x80_8000),
            Exlorom => Some((pc32 & 0x7f_8000) << 1 | pc32 & 0x00_7fff | 0x00_8000),
            Hirom => Some(pc32 | 0x80_0000),
            Exhirom => Some((pc32 & 0x40_0000) << 1 | (pc32 & 0x3f_ffff)),
            _ => unimplemented!(),
        }
    }

    pub fn snes_seg_ofs(&self) -> (u8, u32) {
        unimplemented!()
    }

    pub fn segment(&self) -> Option<u8> {
        match self.map {
            Sa1rom(_) | Sddrom(_) => Some((self.pc / 0x10_0000) as u8),
            _ => None,
        }
    }

    pub fn pc_addr(&self) -> usize {
        self.pc
    }
}

