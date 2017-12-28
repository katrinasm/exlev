use super::{DcErr, DcResult};
use super::lc_lz_shared::{
    parse_command_head, direct_copy, byte_fill, word_fill, remaining_len, repeat
};

pub fn decomp(buf: &[u8]) -> DcResult<Vec<u8>> {
    decomp_with_hint(buf, 4 * 1024)
}

pub fn decomp_with_hint(buf: &[u8], size_hint: usize) -> DcResult<Vec<u8>> {
    let mut input_idx = 0;
    let mut targ = Vec::with_capacity(size_hint);
    while let Some((cmd, len)) = parse_command_head(&buf[input_idx..])? {
        input_idx = match cmd {
            0 => direct_copy(buf, &mut targ, input_idx, len)?,
            1 => byte_fill(buf, &mut targ, input_idx, len)?,
            2 => word_fill(buf, &mut targ, input_idx, len)?,
            3 => inc_fill(buf, &mut targ, input_idx, len)?,
            4 => repeat(buf, &mut targ, input_idx, len)?,
            5 | 6 => return Err(DcErr::LcLzUndefinedLz2Command),
            _ => panic!("incomprehensible failure in LZ2 DC"),
        };
    }
    
    if targ.len() > 0xffff {
        return Err(DcErr::LcLzOverlongOutput);
    };
    
    targ.shrink_to_fit();
    
    Ok(targ)
}

fn inc_fill(buf: &[u8], targ: &mut Vec<u8>, idx: usize, len: u16)
-> DcResult<usize> {
    if remaining_len(buf, idx) < 1 {
        return Err(DcErr::LcLzPrematureTermination);
    }
    
    let mut value = buf[0];
    
    for _ in 0 .. len {
        targ.push(value);
        value = value.wrapping_add(1);
    }
    
    Ok(idx + 1)
}

