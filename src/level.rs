#![allow(dead_code, unused_variables, unused_imports)]
use std::collections::{BTreeSet, BTreeMap};
use std::cmp;
use snes_color::{SnesColor, SnesPal};

use spr::*;
use entrance::{EntrancePlacement, EntranceId};

//#[derive(Debug)]
pub struct PScreen {
    pub tiles: [u16; 256],
    pub sprites: SprSet,
    pub exit: EntranceId,
}

impl Clone for PScreen {
    fn clone(&self) -> PScreen {
        let mut tiles: [u16; 256] = [0; 256];
        for i in 0..256 {
            tiles[i] = self.tiles[i];
        }
        PScreen {
            tiles: tiles,
            sprites: self.sprites.clone(),
            exit: self.exit,
        }
    }
}

impl PScreen {
    fn new() -> PScreen {
        PScreen {
            tiles: [0x0025; 256],
            sprites: SprSet::new(),
            exit: EntranceId::default(),
        }
    }

    fn tile(&self, x: usize, y: usize) -> u16 {
        assert!(x < 16, "tile index out of bounds");
        assert!(y < 16, "tile index out of bounds");
        self.tiles[y * 16 + x]
    }

    fn tile_mut(&mut self, x: usize, y: usize) -> &mut u16 {
        assert!(x < 16, "tile index out of bounds");
        assert!(y < 16, "tile index out of bounds");
        &mut self.tiles[y * 16 + x]
    }
}

// P-Screens aren't Eq because a screen with a sprite cannot be equal
// to any screen, including an otherwise identical screen
impl cmp::PartialEq for PScreen {
    fn eq(&self, other: &PScreen) -> bool {
        if self.sprites.len() == 0 && other.sprites.len() == 0 {
            self.tiles.iter().eq(other.tiles.iter()) && self.exit == other.exit
        } else {
            false
        }
    }
}

pub struct PScrGrid {
    pscreens: Vec<PScreen>,
    width: usize,
    height: usize,
}

impl PScrGrid {
    pub fn new(width: usize, height: usize) -> PScrGrid {
        let w = width;
        let h = height;
        assert!(w < 128, "tried to make layout too wide");
        assert!(h < 128, "tried to make layout too tall");
        assert!(w * h <= 1024, "tried to make layout with too many screens");

        let mut scrs = Vec::with_capacity(w * h);

        for _ in 0..w * h {
            scrs.push(PScreen::new());
        }

        PScrGrid {
            pscreens: scrs,
            width: w,
            height: h,
        }
    }

    pub fn screen_at(&self, x: usize, y: usize) -> &PScreen {
        assert!(x < self.width, "screen index out of bounds");
        assert!(y < self.height, "screen index out of bounds");
        &self.pscreens[x * self.width + y]
    }

    pub fn screen_at_mut(&mut self, x: usize, y: usize) -> &mut PScreen {
        assert!(x < self.width, "screen index out of range");
        assert!(y < self.height, "screen index out of range");
        &mut self.pscreens[y * self.width + x]
    }
}

//#[derive(Debug)]
pub struct ScreenDex {
    pub screens: Vec<PScreen>,
    pub dex: Vec<u8>,
    pub filter: Vec<bool>,
    pub width: usize,
    pub height: usize,
}

impl ScreenDex {
    pub fn empty(width: usize, height: usize) -> ScreenDex {
        assert!(width < 128, "tried to make screendex too wide");
        assert!(height < 128, "tried to make screendex too tall");
        assert!(
            width * height < 1024,
            "tried to make screendex with too many screens"
        );
        ScreenDex {
            screens: vec![],
            dex: vec![],
            filter: vec![],
            width: width,
            height: height,
        }
    }

    pub fn from_scr_grid(mut grid: PScrGrid) -> ScreenDex {
        let mut screens = Vec::with_capacity(128);
        let mut dex = Vec::with_capacity(grid.width * grid.height);

        for (i, scr) in grid.pscreens.drain(..).enumerate() {
            if let Some(pos) = screens.iter().position(|s| *s == scr) {
                dex.push(pos as u8);
            } else {
                let here = screens.len();
                assert!(here < 128, "level has too many distinct screens");
                screens.push(scr);
                dex.push(here as u8);
            }
        }

        ScreenDex {
            screens: screens,
            // these are out of order because `dex: dex` moves dex
            filter: vec![false; dex.len()],
            dex: dex,
            width: grid.width,
            height: grid.width,
        }
    }

    pub fn from_level(level: &Level) -> ScreenDex {
        let mut screens = Vec::with_capacity(128);
        let mut dex = Vec::with_capacity(level.fg.width * level.fg.height * 2);
        let (fg, bg) = (&level.fg.pscreens, &level.bg.pscreens);
        for (i, scr) in fg.iter().chain(bg.iter()).enumerate() {
            if let Some(pos) = screens.iter().position(|s| *s == *scr) {
                dex.push(pos as u8);
            } else {
                let here = screens.len();
                assert!(here < 128, "level has too many distinct screens");
                screens.push(scr.clone());
                dex.push(here as u8);
            }
        }

        ScreenDex {
            screens: screens,
            filter: level.sf.clone(),
            dex: dex,
            width: level.fg.width,
            height: level.fg.height,
        }
    }

    pub fn screen_at(&self, x: usize, y: usize) -> &PScreen {
        assert!(x < self.width, "screen index out of bounds");
        assert!(y < self.height, "screen index out of bounds");
        &self.screens[self.dex[y * self.width + x] as usize]
    }

    pub fn screen_at_mut(&mut self, x: usize, y: usize) -> &mut PScreen {
        assert!(x < self.width, "screen index out of bounds");
        assert!(y < self.height, "screen index out of bounds");
        &mut self.screens[self.dex[y * self.width + x] as usize]
    }

    pub fn tile_bytes(&self) -> Vec<u8> {
        tile_bytes(self.screens.iter())
    }

    pub fn dex_bytes(&self) -> Vec<u8> {
        let mut v = self.dex.clone();
        for (b, &s) in v.iter_mut().zip(self.filter.iter()) {
            if s {
                *b |= 0x80;
            }
        }
        v
    }
}

//#[derive(Debug)]
pub struct Level {
    fg: PScrGrid,
    bg: PScrGrid,
    sf: Vec<bool>,
    pub entrances: Vec<EntrancePlacement>,
    pub header: LevelHeader,
}

impl Level {
    pub fn from_parts(fg: PScrGrid, bg: PScrGrid, sf: Vec<bool>, entrances: Vec<EntrancePlacement>, header: LevelHeader) -> Level {
        assert!(
            fg.width == bg.width && fg.height == bg.height,
            "mismatched FG and BG size"
        );
        assert!(
            fg.width == fg.height && fg.width == 32,
            "unsupported level staging size"
        );
        assert!(fg.width * fg.height == sf.len(), "wrong size scroll filter");
        
        let secondaries = entrances.iter().filter(|en| en.id.secondary).count();
        assert!(
            secondaries <= 32,
            "too many secondary entrances (max is 32)"
        );
        
        let primaries = entrances.iter().filter(|en| !en.id.secondary).count();
        assert!(
            primaries <= 2,
            "too many primary entrances (max is 2)"
        );
        
        for en in &entrances {
            if en.id.secondary {
                assert!(en.id.sub_id < 0x20,
                "too high secondary entrance ID (allowed are 0 ..= 1f)");
            } else {
                assert!(en.id.sub_id < 2,
                "too high primary entrance ID (allowed are 0 ..= 1)");
            }
        }
        
        Level {fg, bg, sf, entrances, header}
    }

    pub fn tile_bytes(&self) -> Vec<u8> {
        tile_bytes(self.fg.pscreens.iter().chain(self.bg.pscreens.iter()))
    }
    
    pub fn width(&self) -> usize {
        self.fg.width
    }
    
    pub fn height(&self) -> usize {
        self.fg.height
    }
}

#[derive(Debug, Clone)]
pub enum Palette {
    Shared(SharedPal),
    Custom(SnesPal),
}

#[derive(Debug, Copy, Clone)]
pub struct SharedPal {
    pub fg: u8,
    pub bg: u8,
    pub sp: u8,
    pub sky: u8,
}

#[derive(Debug, Clone)]
pub struct LevelHeader {
    pub palette: Palette,
    pub mode: u8,
    pub audio_track: u8,
    pub tileset_fg: u8,
    pub tileset_sp: u8,
    pub time: u8,
    pub scroll: u8,
    pub l3_img: u8,
    pub l3_prio: bool,
}

impl ::std::default::Default for LevelHeader {
    // These settings come from Level 105.
    fn default() -> LevelHeader {
        LevelHeader {
            palette: Palette::Shared(SharedPal {fg: 0, bg: 1, sp: 0, sky: 2}),
            mode: 0,
            scroll: 2,
            audio_track: 0x02,
            time: 3,
            tileset_fg: 7,
            tileset_sp: 8,
            l3_img: 0,
            l3_prio: false,
        }
    }
} 

fn tile_bytes<'a, I: Iterator<Item = &'a PScreen>>(it: I) -> Vec<u8> {
    let mut v = Vec::new();
    for scr in it {
        push_scr_bytes(&mut v, scr);
    }
    v
}

fn push_scr_bytes(v: &mut Vec<u8>, scr: &PScreen) {
    for tile in scr.tiles.iter() {
        v.push((*tile & 0xff) as u8);
    }
    for tile in scr.tiles.iter() {
        v.push((*tile >> 8) as u8);
    }
}

pub fn pscreens_from_linear_tiles(tiles: &[u16], width: usize, height: usize) -> PScrGrid {
    let mut scrgrid = PScrGrid::new(width, height);

    let row_width = width as usize * 16;

    for (i, &tile) in tiles.iter().enumerate() {
        let (ax, ay) = (i % row_width, i / row_width);
        let (vx, vy) = (ax % 16, ay % 16);
        let (sx, sy) = (ax / 16, ay / 16);
        *scrgrid.screen_at_mut(sx, sy).tile_mut(vx, vy) = tile;
    }

    scrgrid
}

pub fn place_sprites(scrgrid: &mut PScrGrid, sprs: &SprSet) {
    for spr in sprs {
        let ps = scrgrid.screen_at_mut(spr.scr_x(), spr.scr_y());
        ps.sprites.insert(*spr);
    }
}

pub fn place_exits(scrgrid: &mut PScrGrid, exits: &BTreeMap<(u8, u8), EntranceId>) {
    for iy in 0 .. 32 {
        for ix in 0 .. 32 {
            let ps = scrgrid.screen_at_mut(ix as usize, iy as usize);
            if let Some(&v) = exits.get(&(ix, iy)) {
                ps.exit = v;
            }
        }
    }
}
