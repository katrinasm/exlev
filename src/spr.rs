use std::collections::BTreeSet;
use std::cmp;

#[derive(Copy, Clone, Debug)]
pub struct SpritePlacement {
    pub id: u16,
    pub pos_x: u16,
    pub pos_y: u16,
    pub xbit: bool,
    pub xbytes: [u8; 4],
}

pub type SprSet = BTreeSet<SpritePlacement>;

impl SpritePlacement {
    pub fn new(id: u16, pos_x: u16, pos_y: u16, xbit: bool, xbytes: [u8; 4]) -> SpritePlacement {
        SpritePlacement {
            id: id,
            pos_x: pos_x,
            pos_y: pos_y,
            xbit: xbit,
            xbytes: xbytes,
        }
    }

    pub fn scr_x(&self) -> usize {
        (self.pos_x as usize) / 16
    }
    pub fn scr_y(&self) -> usize {
        (self.pos_y as usize) / 16
    }
    pub fn local_ofs_x(&self) -> usize {
        (self.pos_x as usize) % 16
    }
    pub fn local_ofs_y(&self) -> usize {
        (self.pos_y as usize) % 16
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let long = self.xbytes[1] != 0 || self.xbytes[2] != 0 || self.xbytes[3] != 0;
        let mut out = vec![
            (self.id >> 5 & 0x18) as u8 | if long { 2 } else { 0 } |
                if self.xbit { 4 } else { 0 },
            (self.id & 0xff) as u8,
            ((self.pos_y as u8) << 4) | (self.xbytes[0] & 0xf),
            ((self.pos_x as u8) << 4) | (self.xbytes[0] >> 4),
        ];
        if long {
            out.extend_from_slice(&self.xbytes[1..]);
            out.push(0);
        };
        out
    }
    
    pub const TERMINATOR_BYTES: [u8; 4] = [0x80, 0, 0, 0];
}

impl cmp::Ord for SpritePlacement {
    fn cmp(&self, other: &SpritePlacement) -> cmp::Ordering {
        // The screendex is row-major
        if self.scr_y() != other.scr_y() {
            self.scr_y().cmp(&other.scr_y())
        } else if self.scr_x() != other.scr_x() {
            self.scr_x().cmp(&other.scr_x())
        // Individual screens are column-major
        } else if self.local_ofs_x() != other.local_ofs_x() {
            self.local_ofs_x().cmp(&other.local_ofs_x())
        } else if self.local_ofs_y() != other.local_ofs_y() {
            self.local_ofs_y().cmp(&other.local_ofs_y())
        // we have to complete the equality for Eq,
        // so we decide on a mostly arbitrary ordering for the other elements
        } else if self.id != other.id {
            self.id.cmp(&other.id)
        } else if self.xbit != other.xbit {
            self.xbit.cmp(&other.xbit)
        } else {
            self.xbytes.cmp(&other.xbytes)
        }
    }
}

impl cmp::PartialOrd for SpritePlacement {
    fn partial_cmp(&self, other: &SpritePlacement) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl cmp::PartialEq for SpritePlacement {
    fn eq(&self, other: &SpritePlacement) -> bool {
        self.cmp(other) == cmp::Ordering::Equal
    }
}

impl cmp::Eq for SpritePlacement {}
