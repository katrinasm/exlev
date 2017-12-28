#![allow(dead_code, unused_variables)]
use std::io::{self, SeekFrom};
use level::{Level, ScreenDex, Palette};

use super::EncodeError;
use super::rle;

// This module is not optimized.

pub fn write_level_body<W: io::Write + io::Seek>(
    dest: &mut W,
    level: &Level,
    base_addr: u32,
) -> Result<u32, EncodeError> {
    let ref screendex = ScreenDex::from_level(level);
    // we need to skip 8 bytes as a "hole" for pointers
    let ptr_loc = dest.seek(SeekFrom::Current(0)).unwrap();
    dest.seek(SeekFrom::Current(8)).unwrap();
    
    let screens = base_addr + 8;
    let dex = screens + write_screens(dest, screendex)?;
    let sprites = dex + write_dex(dest, screendex)?;
    let pal = {
        // add a padding byte if the palette is unaligned
        let end_of_sprites = sprites + write_sprites(dest, screendex)?;
        if end_of_sprites & 1 == 1 {
            dest.write_all(&[0]).unwrap();
            end_of_sprites + 1
        } else {
            end_of_sprites
        }
    };
    let entrances = pal + write_pal(dest, level)?;
    let exits = entrances + write_entrances(dest, level)?;
    let header = exits + write_exits(dest, screendex)?;
    let end = header + write_header(dest, level, dex, pal, entrances, exits)?;
    
    print!("start: {:06x}\nscreens: {:06x}\ndex: {:06x}\nsprites: {:06x}\npal: {:06x}\nentrances: {:06x}\nexits: {:06x}\nheader: {:06x}\nend: {:06x}\n",
        base_addr, screens, dex, sprites, pal, entrances, exits, header, end
    );
    
    dest.seek(SeekFrom::Start(ptr_loc)).unwrap();
    write_long(dest, sprites)?;
    write_long(dest, header)?;
    
    Ok(end - base_addr)
}

fn write_dex<W: io::Write>(dest: &mut W, dex: &ScreenDex) -> Result<u32, EncodeError> {
    let bytes = dex.dex_bytes();
    dest.write_all(&bytes).unwrap();
    Ok(bytes.len() as u32)
}

fn write_screens<W: io::Write>(dest: &mut W, dex: &ScreenDex) -> Result<u32, EncodeError> {
    let tile_bytes = dex.tile_bytes();
    let mut buf: [u8; 3];
    let mut length = 0;
    // this could plausibly be split off into a function,
    // but would require a lot of intermediate types for iterators.
    for run in rle::make_runs(tile_bytes.iter().map(|v| *v)) {
        buf = [run.length as u8, (run.length >> 8) as u8, run.val];
        dest.write_all(&buf).unwrap(); // this is not great
        length += buf.len() as u32;
    }
    dest.write_all(&[0, 0]).unwrap();
    length += 2;
    Ok(length)
}

fn write_sprites<W: io::Write>(dest: &mut W, dex: &ScreenDex) -> Result<u32, EncodeError> {
    let terminator = &::spr::SpritePlacement::TERMINATOR_BYTES;
    // This size guess is precisely the maximum.
    let mut spriteofs = Vec::with_capacity(0x100);
    // This size guess is enough for exactly one sprite per screen.
    let mut spritebin = Vec::with_capacity(dex.screens.len() * 4);
    spritebin.extend_from_slice(terminator);

    let mut spriteind = 0;
    for scr in dex.screens.iter() {
        if scr.sprites.len() == 0 {
            spriteofs.extend_from_slice(&[0, 0]);
        } else {
            spriteofs.push(((spritebin.len() / 4) as u8) << 1);
            spriteofs.push(spriteind);
            for spr in scr.sprites.iter() {
                spritebin.extend(&spr.to_bytes());
                spriteind += 1;
            }
            spritebin.extend(terminator);
        }
    }

    spriteofs.resize(0x100, 0);

    dest.write_all(&[0]).unwrap();
    dest.write_all(spriteofs.as_slice()).unwrap();
    dest.write_all(spritebin.as_slice()).unwrap();
    Ok(spriteofs.len() as u32 + spritebin.len() as u32 + 1)
}

fn write_pal<W: io::Write>(dest: &mut W, level: &Level) -> Result<u32, EncodeError> {
    if let Palette::Custom(ref pal) = level.header.palette {
        for color in pal.iter() {
            let w = color.to_snes();
            let b = [w as u8, (w >> 8) as u8];
            dest.write_all(&b).unwrap();
        }
        Ok(0x202)
    } else {
        Ok(0)
    }
}

fn write_entrances<W: io::Write>(dest: &mut W, level: &Level) -> Result<u32, EncodeError> {
    let ens = {
        let mut ens = level.entrances.clone();
        ens.sort();
        ens
    };
    let primaries = ens.iter().filter(|en| !en.id.secondary).count();
    dest.write_all(&[primaries as u8]).unwrap();
    let mut len = 1;
    
    for en in ens {
        len += write_entrance(
            dest,
            en.id.levelnum,
            en.pos_x, en.pos_y,
            en.anim,
            0, // bgofs
            0, // scroll
            3, // three
            false, // intro
            en.water, en.slippery
        )?;
    }
    
    Ok(len)
}

fn write_entrance<W: io::Write>(
    dest: &mut W,
    lvlnum: u16,
    x: u16, y: u16,
    anim: u8,
    bgofs: u16,
    scroll: u8,
    three: u8,
    intro: bool,
    wet: bool, slip: bool,
) -> Result<u32, EncodeError> {
    dest.write_all(&[
        lvlnum as u8,
        ((lvlnum >> 8) as u8) | (anim << 1) | ((x << 4) as u8),
        ((x >> 4) as u8) | ((bgofs << 5) as u8),
        ((bgofs >> 3 & 0xf) as u8) | ((y << 4) as u8),
        ((y >> 4) as u8) | ((intro as u8) << 5) | (three << 6),
        (scroll as u8) | ((bgofs >> 3 & 0x30) as u8) | ((wet as u8) << 6) | ((slip as u8) << 7),
    ]).unwrap();
    Ok(6)
}

fn write_exits<W: io::Write>(dest: &mut W, dex: &ScreenDex) -> Result<u32, EncodeError> {
    for (i,scr) in dex.screens.iter().enumerate() {
        println!("{:02x} -> {}", i, scr.exit);
        dest.write_all(&scr.exit.to_bytes()).unwrap();
    }
    Ok((dex.screens.len() as u32) * 3)
}

fn write_header<W: io::Write>(
    dest: &mut W,
    level: &Level,
    dex_addr: u32,
    pal_addr: u32,
    entrance_addr: u32,
    exit_addr: u32,
) -> Result<u32, EncodeError> {
    write_long(dest, dex_addr)?;
    write_long(dest, exit_addr)?;
    dest.write_all(&[
        32, // width
        32, // height
        (level.header.l3_img << 5) | level.header.mode,
    ]).unwrap();
    if let Palette::Shared(p) = level.header.palette {
        dest.write_all(&[0, p.sp | (p.sky << 3), p.bg | (p.fg << 3)]).unwrap()
    } else {
        write_long(dest, pal_addr | 1)?;
    }
    dest.write_all(&[
        level.header.audio_track,
        (level.header.tileset_sp << 4) | level.header.tileset_fg,
        (level.header.time << 4) | ((level.header.l3_prio as u8) << 3) | level.header.scroll as u8,
    ]).unwrap();
    write_long(dest, entrance_addr)?;
    Ok(17)
}

fn write_long<W: io::Write>(dest: &mut W, v: u32) -> Result<(), EncodeError> {
    Ok(dest.write_all(
        &[v as u8, (v >> 8) as u8, (v >> 16) as u8]
    ).unwrap())
}

