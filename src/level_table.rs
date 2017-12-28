use super::address::Address;
use super::address::Mapper::*;

pub fn get_level_ptr<B: AsRef<[u8]>>(rombytes: &B, level: u16) -> Address {
    let rb = rombytes.as_ref();
    let ptr_ofs = get_level_ptr_ofs(level).pc_ofs();
    let ptr = Address::new_from_snes_bytes(&rb[ptr_ofs .. ptr_ofs + 3], Lorom).unwrap();
    ptr
}

pub fn set_level_ptr(rombytes: &mut [u8], level: u16, value: u32) {
    let ptr_ofs = get_level_ptr_ofs(level).pc_ofs();
    rombytes[ptr_ofs] = value as u8;
    rombytes[ptr_ofs + 1] = (value >> 8) as u8;
    rombytes[ptr_ofs + 2] = (value >> 16) as u8;
}

fn get_level_ptr_ofs(level: u16) -> Address {
    assert!(level < 0x200, "tried to access level with too high level number");
    let addr = 0x05_e000 + level as usize * 3;
    Address::new_from_snes(addr, Lorom).unwrap()
}

pub fn rm_level(rombytes: &mut [u8], level: u16) -> Option<(Address, usize)> {
    let start = get_level_ptr(&rombytes, level).pc_ofs();
    if start < 12 {
        return None;
    };
    let tag_start = start - 12;
    if !is_rats_clnp(&rombytes[tag_start .. start]) {
        return None;
    };
    
    let len = (rombytes[tag_start + 4] as usize | ((rombytes[tag_start + 5] as usize) << 8)) + 8;
    
    let end = if tag_start + len > rombytes.len() { rombytes.len() } else { tag_start + len };
    
    for i in tag_start .. end {
        rombytes[i] = 0;
    }
    
    set_level_ptr(rombytes, level, 0);
    
    let new_ptr = Address::new_from_pc(tag_start, Lorom).unwrap();
    Some((new_ptr, len))
}

fn is_rats_clnp(bytes: &[u8]) -> bool {
    let b = bytes.as_ref();
    if b.len() < 12 {
        false
    } else {
        b[0..4] == b"STAR"[..]
        && b[4] == !b[6] && b[5] == !b[7]
        && b[8..12] == b"CLNP"[..]
    }
}

