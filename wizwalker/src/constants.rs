<<<<<<< HEAD
// Number of units covered in 1 second
pub const WIZARD_SPEED: u32 = 580;

=======
>>>>>>> origin/port-wizwalker-memory-objects-6518915373428707039
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
<<<<<<< HEAD
=======

impl Primitive {
    pub fn size(&self) -> usize {
        match self {
            Self::Bool => 1,
            Self::Char => 1,
            Self::Int8 => 1,
            Self::Uint8 => 1,
            Self::Int16 => 2,
            Self::Uint16 => 2,
            Self::Int32 => 4,
            Self::Uint32 => 4,
            Self::Int64 => 8,
            Self::Uint64 => 8,
            Self::Float32 => 4,
            Self::Float64 => 8,
        }
    }
}
>>>>>>> origin/port-wizwalker-memory-objects-6518915373428707039
