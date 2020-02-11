pub const MAGIC_NUMBER: &'static str = "CDF";

pub const ZERO: u8         = 0x00000000;
pub const NC_DIMENSION: u8 = 0x0000000a;
pub const NC_VARIABLE: u8  = 0x0000000b;
pub const NC_ATTRIBUTE: u8 = 0x0000000c;

pub const NC_BYTE: u8      = 0x00000001;
pub const NC_CHAR: u8      = 0x00000002;
pub const NC_SHORT: u8     = 0x00000003;
pub const NC_INT: u8       = 0x00000004;
pub const NC_FLOAT: u8     = 0x00000005;
pub const NC_DOUBLE: u8    = 0x00000006;

pub const FILL_CHAR: u8    = 0x00;
pub const FILL_BYTE: u8    = 0x81;
pub const FILL_SHORT: u16  = 0x8001;
pub const FILL_INT: u32    = 0x80000001;
pub const FILL_FLOAT: u32  = 0x7cf00000;
pub const FILL_DOUBLE: u64 = 0x479e000000000000;

pub const STREAMING: u32 = 0xffffffff;
