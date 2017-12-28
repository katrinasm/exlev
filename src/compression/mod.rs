mod lc_lz_shared;
mod compression_error;


pub use self::compression_error::DcErr as DcErr;
pub use self::compression_error::CErr as CErr;

type DcResult<V> = Result<V, DcErr>;
type CResult<V> = Result<V, CErr>;

pub mod lc_lz3;
pub mod lc_lz2;
