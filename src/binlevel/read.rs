use std::io;
use level::Level;

use super::DecodeError;

pub fn read_level<R: io::Read>(source: &mut R) -> Result<Level, DecodeError> {
    let _s = source;
    unimplemented!();
}
