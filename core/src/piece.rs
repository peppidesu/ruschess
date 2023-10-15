#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PieceKind {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PieceColor {
    Black,
    White,
}

impl PieceColor {
    pub fn opposite(&self) -> Self {
        match self {
            Self::Black => Self::White,
            Self::White => Self::Black,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Piece {
    kind: PieceKind,
    color: PieceColor,
}

impl Piece {
    #[cfg(not(tarpaulin_include))]
    pub fn new(kind: PieceKind, color: PieceColor) -> Self {
        Self { kind, color }
    }
    #[cfg(not(tarpaulin_include))]
    pub fn kind(&self) -> PieceKind {
        self.kind
    }
    #[cfg(not(tarpaulin_include))]
    pub fn color(&self) -> PieceColor {
        self.color
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_piece_color_opposite() {
        assert_eq!(PieceColor::Black.opposite(), PieceColor::White);
        assert_eq!(PieceColor::White.opposite(), PieceColor::Black);
    }
}
