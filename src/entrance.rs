#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct EntranceId {
    pub secondary: bool,
    pub levelnum: u16,
    pub sub_id: u8,
    _construct: (),
}

impl EntranceId {
    pub fn main(levelnum: u16) -> Option<EntranceId> {
        if levelnum < 0x200 {
            Some(EntranceId {levelnum, sub_id: 0, secondary: false, _construct: ()})
        } else {
            None
        }
    }
    
    pub fn from_parts(levelnum: u16, sub_id: u8, secondary: bool) -> Option<EntranceId> {
        if levelnum >= 0x200 { // only 0x200 levels
            None
        } else if !secondary && sub_id > 0x10 { // only 0x10 main entrances per level
            None
        } else if sub_id > 0x20 { // only 0x20 secondary entrances per level
            None
        } else {
            Some(EntranceId {levelnum, sub_id, secondary, _construct: ()})
        }
    }
    
    pub fn from_name(name: &str) -> Option<EntranceId> {
        // an entrance name is like
        // "{levelnumber: hex u16}#{fragment}"
        // and the fragment is like
        // "{kind: 'm' for main or 's' for secondary}{sub_id: hex u8}"
        // this function breaks it apart into a level number and fragment
        // and then calls from_num_and_fragment to do the rest
        let mut pieces = name.splitn(1, '#');
        let lvl_piece = if let Some(p) = pieces.next() { p } else { return None };
        let frag_piece = if let Some(p) = pieces.next() { p } else { return None };
        let lvlnum = if let Ok(n) = u16::from_str_radix(lvl_piece, 16) {
            n
        } else {
            return None;
        };
        
        EntranceId::from_num_and_fragment(lvlnum, frag_piece)
    }
    
    pub fn from_num_and_fragment(num: u16, fragment: &str) -> Option<EntranceId> {
        if num > 0x200 {
            return None;
        }
        
        // shortest valid fragment is 2 chars e.g. "m0"
        if fragment.len() < 2 {
            return None;
        }
        
        // get the first byte of the fragment (its kind)
        // unwrap is safe by the length of the fragment
        let kind = fragment.bytes().next().unwrap();
        
        // convert kind letter to secondary flag
        let secondary = if kind == b'm' {
            false
        } else if kind == b's' {
            true
        } else {
            return None
        };
        
        // skip the first char of the string
        // we know the second byte of the fragment is a codepoint start b/c
        // the first byte is either b'm' or b's' if we got here
        let (_, sub_id_s) = fragment.split_at(1);
        
        // convert the string we got to a number
        let sub_id = if let Ok(n) = u8::from_str_radix(sub_id_s, 16) {
            n
        } else {
            return None
        };

        EntranceId::from_parts(num, sub_id, secondary)
    }

    pub fn to_bytes(&self) -> [u8; 3] {
        [
            self.levelnum as u8,
            (self.levelnum >> 8) as u8,
            self.sub_id | (if self.secondary {0x80} else {0}),
        ]
    }
}

impl ::std::default::Default for EntranceId {
    fn default() -> EntranceId {
        EntranceId {secondary: false, levelnum: 0, sub_id: 0, _construct: ()}
    }
}

impl ::std::fmt::Display for EntranceId {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "{:03x}#{}{:02x}",
            self.levelnum,
            if self.secondary {'s'} else {'m'},
            self.sub_id,
        )
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct EntrancePlacement {
    pub id: EntranceId,
    pub pos_y: u16,
    pub pos_x: u16,
    pub anim: u8,
    pub water: bool,
    pub slippery: bool,
    _construct: (),
}

impl EntrancePlacement {
    pub fn new(id: EntranceId, pos_x: u16, pos_y: u16, anim: u8, slippery: bool, water: bool) -> EntrancePlacement {
        EntrancePlacement {id, pos_x, pos_y, anim, slippery, water, _construct: ()}
    }
    
    pub fn from_parts(levelnum: u16, sub_id: u8, secondary: bool, pos_x: u16, pos_y: u16, anim: u8, slippery: bool, water: bool)
    -> Option<EntrancePlacement> {
        if let Some(id) = EntranceId::from_parts(levelnum, sub_id, secondary) {
            Some(EntrancePlacement {id, pos_x, pos_y, anim, slippery, water, _construct: ()})
        } else {
            None
        }
    }
    
    pub fn from_name(name: &str, pos_x: u16, pos_y: u16, anim: u8, slippery: bool, water: bool) -> Option<EntrancePlacement> {
        if let Some(id) = EntranceId::from_name(name) {
            Some(EntrancePlacement {id, pos_x, pos_y, anim, slippery, water, _construct: ()})
        } else {
            None
        }
    }
    
    pub fn from_num_and_fragment(levelnum: u16, fragment: &str, pos_x: u16, pos_y: u16, anim: u8, slippery: bool, water: bool)
    -> Option<EntrancePlacement> {
        if let Some(id) = EntranceId::from_num_and_fragment(levelnum, fragment) {
            Some(EntrancePlacement {id, pos_x, pos_y, anim, slippery, water, _construct: ()})
        } else {
            None
        }
    }
}

