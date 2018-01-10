//! # BEEP FRICKIN BOOP
//! AAAAAAA

extern crate sxd_document;
extern crate sxd_xpath;
pub mod why_sxd;
pub mod spr;
pub mod entrance;
pub mod tmx;
pub mod binlevel;
pub mod level;
pub mod address;
pub mod level_table;
pub mod snes_color;
pub mod compression;
pub mod rats;

use std::io::{Cursor, SeekFrom};
use std::io::prelude::*;

use std::path::PathBuf;
use std::fs::{File, OpenOptions};

fn main() {
    match submain() {
        Err(e) => println!("Error: {}", e),
        _ => (),
    }
}

#[derive(Clone, Debug)]
struct Arguments {
    rom_path: PathBuf,
    item_path: PathBuf,
    action: CliAction,
}

#[derive(Clone, Debug)]
enum CliAction {
	InsertGfx(Option<u16>),
	ExtractGfx(Option<u16>),
	InsertTmx(u16),
	ExtractTmx(u16),
}

macro_rules! try_hex_arg {
    ($s:expr) => {
        if let Ok(v) = u16::from_str_radix($s, 16) {
            v
        } else {
            return None;
        }
    };
}

fn parse_arguments() -> Option<Arguments> {
    let mut rom_path = None;
    let mut item_path = None;
    let mut action = None;

    for arg in std::env::args().skip(1) {
        if arg.starts_with("--rom=") {
            if !rom_path.is_none() { return None; };
            rom_path = Some(PathBuf::from(&arg["--rom=".len()..]));
        } else if arg.starts_with("--insert-tmx=") {
            if !action.is_none() { return None; };
            let num_s = &arg["--insert-tmx=".len()..];
            let lnum = try_hex_arg!(num_s);
            action = Some(CliAction::InsertTmx(lnum));
        } else if arg.starts_with("--extract-tmx=") {
            if !action.is_none() { return None; };
            let num_s = &arg["--extract-tmx=".len()..];
            let lnum = try_hex_arg!(num_s);
            action = Some(CliAction::ExtractTmx(lnum));
        } else if arg.starts_with("--extract-gfx=") {
            if !action.is_none() { return None; };
            let num_s = &arg["--extract-gfx=".len()..];
            let gnum = try_hex_arg!(num_s);
            action = Some(CliAction::ExtractGfx(Some(gnum)));
        } else if arg.starts_with("--insert-gfx=") {
            if !action.is_none() { return None; };
            let num_s = &arg["--insert-gfx=".len()..];
            let gnum = try_hex_arg!(num_s);
            action = Some(CliAction::InsertGfx(Some(gnum)));
        } else if arg.starts_with("--extract-gfx") {
            if !action.is_none() { return None; };
            action = Some(CliAction::ExtractGfx(None))
        } else if arg.starts_with("--insert-gfx") {
            if !action.is_none() { return None; };
            action = Some(CliAction::InsertGfx(None));
        } else {
            if !item_path.is_none() { return None; };
            item_path = Some(PathBuf::from(arg));
        };
    }

    if action.is_some() && rom_path.is_some() && item_path.is_some() {
        Some(Arguments {
            rom_path: rom_path.unwrap(),
            item_path: item_path.unwrap(),
            action: action.unwrap(),
        })
    } else {
        None
    }
}

fn submain() -> Result<(), Box<std::error::Error>> {
    let args = if let Some(a) = parse_arguments() {
        a
    } else {
        panic!("you goofed it on the command line see the readme");
    };
    println!("{:?}", args);

    let mut rom = OpenOptions::new().read(true).write(true).open(args.rom_path)?;

    // 4 MB is a nice enough guess.
    let mut rombytes = Vec::with_capacity(4 * 1024 * 1024);

    rom.read_to_end(&mut rombytes)?;

    match args.action {
        CliAction::InsertTmx(lvln) =>
            rombytes = insert_level(rombytes, lvln, &args.item_path)?,
        CliAction::ExtractTmx(_lvln) =>
            unimplemented!(),
        CliAction::InsertGfx(_) =>
            unimplemented!(),
        CliAction::ExtractGfx(_) =>
            unimplemented!(),
    }

    rom.seek(SeekFrom::Start(0))?;
    rom.write_all(&rombytes)?;

    Ok(())
}

fn insert_level(mut rombytes: Vec<u8>, lvlnum: u16, path: &PathBuf)
-> Result<Vec<u8>, Box<std::error::Error>> {
    let space = rats::find_free(&rombytes, 0x8000).expect("Need free bank");
     // points just past RATS_CLNP tag
    let start = address::Address::new_from_pc(
        space.pc_ofs() + 12, address::Mapper::Lorom
    ).unwrap();

    let mut f = File::open(path)?;
    let lvl = tmx::read_level(&mut f, "lev", lvlnum)?;

    println!("{:?}", level_table::rm_level(&mut rombytes, lvlnum));

    let mut romcur = Cursor::new(rombytes);
    romcur.seek(SeekFrom::Start(start.pc_ofs() as u64))?;

    let start_ptr = start.snes_ofs().unwrap();

    let len = binlevel::write_level_body(&mut romcur, &lvl, start_ptr as u32)?;

    println!("level is {}kB", len / 1024);

    let (len_lo, len_hi) = (len as u8, (len >> 8) as u8);

    romcur.seek(SeekFrom::Start(space.pc_ofs() as u64))?;
    romcur.write_all(&b"STAR"[..])?;
    romcur.write_all(&[len_lo, len_hi, !len_lo, !len_hi])?;
    romcur.write_all(&b"CLNP"[..])?;

    let mut rombytes = romcur.into_inner();

    level_table::set_level_ptr(rombytes.as_mut_slice(), lvlnum, start_ptr);

    Ok(rombytes)
}

