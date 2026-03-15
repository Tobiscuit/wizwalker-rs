// Number of speed units covered per second by a Wizard101 character
pub const WIZARD_SPEED: u32 = 580;

/// Primitive type discriminants, used for typed memory reads
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Primitive {
    Bool,
    Char,
    Int8,
    Uint8,
    Int16,
    Uint16,
    Int32,
    Uint32,
    Int64,
    Uint64,
    Float32,
    Float64,
}
