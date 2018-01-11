use address::{Address, Mapper};

pub fn find_free(rombytes: &[u8], len: usize) -> Option<Address> {
    find_aligned(rombytes, len, 0)
}

pub fn find_aligned(rombytes: &[u8], len: usize, align: u8) -> Option<Address> {
    assert!(len <= 0x8000, "too long freespace to exist");
    assert!(align < 17, "too high alignment for freespace search");

    let begin = 0x10 * 0x8000;

    let filt = (1 << align) - 1;
    let mut i = begin;
    let mut free_block = 0;

    while i < rombytes.len() {
        if let Some(size) = rats_len(&rombytes[i ..]) {
            free_block = 0;
            i += size;
        } else {
            free_block += 1;
            if free_block >= len && i & filt == 0 {
                break;
            }
            i += 1;
            // reset on bank boundaries
            if i & 0x7fff == 0 {
                free_block = 0;
            }
        }
    }
    if free_block >= len {
        Address::new_from_pc(i, Mapper::Lorom)
    } else {
        None
    }
}

pub fn insert<W: ::std::io::Write>(buf: &mut W, data: &[u8]) {
    assert!(data.len() <= 0x1_0000, "tried to insert too large (>64KiB) object");
    assert!(data.len() != 0, "tried to insert zero-length object");
    let len = data.len() as u16;
    let tag = &[
        b'S', b'T', b'A', b'R',
        len as u8, (len >> 8) as u8,
        (!len) as u8, (!len >> 8) as u8,
    ];
    buf.write_all(tag).unwrap();
    buf.write_all(data).unwrap();
}

pub fn insert_free(rombytes: &mut Vec<u8>, data: &[u8]) -> Option<Address> {
    let block = data.len() + 8; // we need 8 extra bytes for the RATS itself
    if let Some(a) = find_free(&*rombytes, block) {
        let ofs = a.pc_ofs();
        // Write is only impl'd for `&mut [u8]`, and we need &mut (something with Write)
        insert(&mut &mut rombytes[ofs .. ofs + block], data);
        Some(a)
    } else {
        None
    }
}

fn rats_len(buf: &[u8]) -> Option<usize> {
    if buf.len() < 8
    || !buf.starts_with(b"STAR")
    || read_u16(&buf[4..]) != !read_u16(&buf[6..]) {
        None
    } else {
        // + 8 for the length of the tag, + 1 since the tag stores len - 1
        Some((read_u16(buf) as usize) + 8 + 1)
    }
}

fn read_u16(buf: &[u8]) -> u16 {
    assert!(buf.len() >= 2, "tried to read u16 from too-short buffer");
    ((buf[0] as u16) | ((buf[1] as u16) << 8))
}

