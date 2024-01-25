
macro_rules! piece {
    ($kind:ident, $color:ident) => {
        Piece::new(PieceKind::$kind, PieceColor::$color)
    };
}
