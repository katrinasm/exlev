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
            3 => zero_fill(buf, &mut targ, input_idx, len)?,
            4 => repeat(buf, &mut targ, input_idx, len)?,
            5 => repeat_bit_reversed(buf, &mut targ, input_idx, len)?,
            6 => repeat_backward(buf, &mut targ, input_idx, len)?,
            _ => panic!("incomprehensible failure in LZ3 DC"),
        };
    }

    if targ.len() > 0xffff {
        return Err(DcErr::LcLzOverlongOutput);
    };

    targ.shrink_to_fit();

    Ok(targ)
}


fn zero_fill(buf: &[u8], targ: &mut Vec<u8>, idx: usize, len: u16)
-> DcResult<usize> {
    if remaining_len(buf, idx) < 1 {
        return Err(DcErr::LcLzPrematureTermination);
    }

    for _ in 0 .. len {
        targ.push(0);
    }

    Ok(idx)
}

fn repeat_bit_reversed(buf: &[u8], targ: &mut Vec<u8>, idx: usize, len: u16)
-> DcResult<usize> {
    if remaining_len(buf, idx) < 2 {
        return Err(DcErr::LcLzPrematureTermination);
    }

    let ofs = buf[idx] as usize | ((buf[idx + 1] as usize) << 1); // offset is big-endian fsr

    if ofs >= targ.len() {
        return Err(DcErr::LcLzOutOfRangeCopy);
    }

    let rev_nybbles = [
        0b0000, 0b1000, 0b0100, 0b1100,
        0b0010, 0b1010, 0b0110, 0b1110,
        0b0001, 0b1001, 0b0101, 0b1101,
        0b0011, 0b1011, 0b0111, 0b1111,
    ];

    for i in ofs .. ofs + len as usize {
        let b = targ[i];
        let (ln, hn) = (b & 0xf, b >> 4);
        let (rln, rhn) = (rev_nybbles[ln as usize], rev_nybbles[hn as usize]);
        let rb = rhn | (rln << 4);
        targ.push(rb);
    }

    Ok(idx + 2)
}

fn repeat_backward(buf: &[u8], targ: &mut Vec<u8>, idx: usize, len: u16)
-> DcResult<usize> {
    if remaining_len(buf, idx) < 2 {
        return Err(DcErr::LcLzPrematureTermination);
    }

    let ofs = buf[idx] as usize | ((buf[idx + 1] as usize) << 1); // offset is big-endian fsr

    if ofs >= targ.len() {
        return Err(DcErr::LcLzOutOfRangeCopy);
    }

    // the last byte copied is ofs - len
    if (len as usize) < ofs {
        return Err(DcErr::LcLzOutOfRangeCopy);
    }

    for i in ofs .. ofs - len as usize {
        let b = targ[i];
        targ.push(b);
    }

    Ok(idx + 2)
}

