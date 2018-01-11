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
    set_version(rombytes, level);

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

const VERSION_PTR_LOC: usize = 0x7f080;

pub fn get_version(rombytes: &[u8], lvlnum: u16) -> (u8, u8, u8) {
    if let Some(table_ptr) = get_version_table_ptr(rombytes) {
        let ofs = table_ptr.pc_ofs() + (lvlnum as usize) * 3;
        (rombytes[ofs], rombytes[ofs + 1], rombytes[ofs + 2])
    } else {
        (0, 0, 0)
    }
}

pub fn set_version(rombytes: &mut [u8], lvlnum: u16) -> Result<(), String> {
    let level_version = [0, 1, 0];
    let table_ptr = match get_version_table_ptr(rombytes) {
        None =>
            init_version_table(rombytes).ok_or("no dang space for the level table")?,
        Some(a) => a,
    };

    println!("ver table: {:08x}", table_ptr.pc_ofs());

    let targ = table_ptr.pc_ofs() + (lvlnum as usize) * 3;
    rombytes[targ] = level_version[0];
    rombytes[targ + 1] = level_version[1];
    rombytes[targ + 2] = level_version[2];
    Ok(())
}

fn get_version_table_ptr(rombytes: &[u8]) -> Option<Address> {
    let a = Address::new_from_snes_bytes(
        &rombytes[VERSION_PTR_LOC .. VERSION_PTR_LOC + 3],
        Lorom
    ).unwrap();
    if a.snes_ofs().unwrap() == 0xff_ffff {
        None
    } else {
        Some(a)
    }
}

fn init_version_table(rombytes: &mut [u8]) -> Option<Address> {
    let default = &[0; 512 * 3];
    let a = match ::rats::insert_free(rombytes, default) {
        None => return None,
        Some(a) => a,
    };
    let ofs = a.snes_ofs().unwrap();
    rombytes[VERSION_PTR_LOC] = ofs as u8;
    rombytes[VERSION_PTR_LOC + 1] = (ofs >> 8) as u8;
    rombytes[VERSION_PTR_LOC + 2] = (ofs >> 16) as u8;
    Some(a)
}

