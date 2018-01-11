use super::{DcErr, DcResult};

pub fn parse_command_head(buf: &[u8]) -> DcResult<Option<(u8, u16)>> {
    if buf.len() == 0 {
        return Err(DcErr::LcLzPrematureTermination);
    };

    // end-of-stream
    if buf[0] == 0xff {
        return Ok(None)
    }

    // short cmd
    let (cmd, len) = (buf[0] >> 5 & 0x7, buf[0] as u16 & 0x1f);

    if cmd != 7 {
        // short command
        Ok(Some((cmd, len + 1)))
    } else if buf.len() >= 2 {
        // long command
        let (cmd, len) = (
            buf[0] >> 2 & 0x7,
            ((buf[0] as u16 & 0x2) << 8) | buf[1] as u16,
        );
        if cmd == 7 {
            // double-long command; invalid
            Err(DcErr::LcLzInvalidCommandHeader)
        } else {
            Ok(Some((cmd, len + 1)))
        }
    } else {
        // premature-terminated long command
        Err(DcErr::LcLzPrematureTermination)
    }
}

pub fn remaining_len(buf: &[u8], idx: usize) -> usize {
    assert!(idx < buf.len(), "attempted out-of-bounds read in decompression");
    buf.len() - idx
}

pub fn direct_copy(buf: &[u8], targ: &mut Vec<u8>, idx: usize, len: u16)
-> DcResult<usize> {
    let uslen = len as usize;
    if uslen >= remaining_len(buf, idx) {
        return Err(DcErr::LcLzPrematureTermination);
    }

    for &byte in buf[idx .. idx + uslen].iter() {
        targ.push(byte);
    }

    Ok(idx + uslen as usize)
}

pub fn byte_fill(buf: &[u8], targ: &mut Vec<u8>, idx: usize, len: u16)
-> DcResult<usize> {
    if remaining_len(buf, idx) < 1 {
        return Err(DcErr::LcLzPrematureTermination);
    }

    for _ in 0 .. len {
        targ.push(buf[idx]);
    }

    Ok(idx + 1)
}

pub fn word_fill(buf: &[u8], targ: &mut Vec<u8>, idx: usize, len: u16)
-> DcResult<usize> {
    if remaining_len(buf, idx) < 2 {
        return Err(DcErr::LcLzPrematureTermination);
    }

    let mut lo_hi = 0;
    for _ in 0 .. len {
        targ.push(buf[idx + lo_hi]);
        lo_hi ^= 1;
    }

    Ok(idx + 2)
}

pub fn repeat(buf: &[u8], targ: &mut Vec<u8>, idx: usize, len: u16)
-> DcResult<usize> {
    if remaining_len(buf, idx) < 2 {
        return Err(DcErr::LcLzPrematureTermination);
    }

    let ofs = buf[idx] as usize | ((buf[idx + 1] as usize) << 1); // offset is big-endian fsr

    if ofs >= targ.len() {
        return Err(DcErr::LcLzOutOfRangeCopy);
    }

    for i in ofs .. ofs + len as usize {
        let b = targ[i];
        targ.push(b);
    }

    Ok(idx + 2)
}

