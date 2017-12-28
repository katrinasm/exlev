
use std::io;
use level::Level;

pub fn write_level<W: io::Write>(dest: &mut W, level: &Level) -> io::Result<()> {
    let (_d, _l) = (dest, level);

    unimplemented!();
}
