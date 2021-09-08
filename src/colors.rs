// type alias for colors
pub type Color = [u8; 4];

// consts for colors
pub(crate) const WHITE: Color = [0xff, 0xff, 0xff, 0xff]; // FFFFFF
pub(crate) const BLACK: Color = [0x00, 0x00, 0x00, 0xff]; // 000000
pub(crate) const DARK_GREEN: Color = [0x20, 0x2a, 0x25, 0xff]; // 202A25
pub(crate) const GRAY: Color = [0xeB, 0xe9, 0xe9, 0xff]; //EBE9E9
pub(crate) const GREEN: Color = [0x00, 0xA8, 0x78, 0xff]; // 00A878
pub(crate) const YELLOW: Color = [0xf8, 0xf3, 0x2b, 0xff]; // F8F32B
pub(crate) const BLUE: Color = [0x6c, 0xcf, 0xf6, 0xff]; // 6CCFF6
pub(crate) const FLAME: Color = [0xcf, 0x5c, 0x36, 0xff]; // CF5C36

pub(crate) const GRID_LINES: Color = GRAY;
pub(crate) const GRID_EMPTY: Color = BLUE;
pub(crate) const BACKGROUND: Color = DARK_GREEN;
