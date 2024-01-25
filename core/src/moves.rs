use crate::{
    board::{Board, Square},
    piece::{Piece, PieceColor, PieceKind},
    position::Position,
    state::{CastleSide, GameState},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Move {
    Normal {
        from: Position,
        to: Position,
    },
    Capture {
        from: Position,
        to: Position,
        captured: Piece,
    },
    EnPassant {
        from: Position,
        to: Position,
        captured: Position,
    },
    DoublePawnPush {
        from: Position,
        to: Position,
        en_passant: Position,
    },
    Promotion {
        from: Position,
        to: Position,
        promoted: Piece,
    },
    PromotionCapture {
        from: Position,
        to: Position,
        captured: Piece,
        promoted: Piece,
    },
    Castle {
        from: Position,
        to: Position,
        rook_from: Position,
        rook_to: Position,
    },
}

pub struct PawnMoveInfo {
    pub color: PieceColor,
    pub start_rank: u8,
    pub promotion_rank: u8,
    pub push_offset: i8,
    pub double_push_offset: i8,
}
impl PawnMoveInfo {
    pub fn new(color: PieceColor) -> Self {
        match color {
            PieceColor::White => Self {
                color: PieceColor::White,
                start_rank: 1,
                promotion_rank: 7,
                push_offset: 8,
                double_push_offset: 16,
            },
            PieceColor::Black => Self {
                color: PieceColor::Black,
                start_rank: 6,
                promotion_rank: 0,
                push_offset: -8,
                double_push_offset: -16,
            },
        }
    }
    #[inline]
    pub fn push(&self, position: Position) -> Position {
        ((u8::from(position) as i8 + self.push_offset) as u8).into()
    }
    #[inline]
    pub fn double_push(&self, position: Position) -> Position {
        ((u8::from(position) as i8 + self.double_push_offset) as u8).into()
    }
    #[inline]
    pub fn left_capture(&self, position: Position) -> Position {
        ((u8::from(position) as i8 + self.push_offset - 1) as u8).into()
    }
    #[inline]
    pub fn right_capture(&self, position: Position) -> Position {
        ((u8::from(position) as i8 + self.push_offset + 1) as u8).into()
    }
    #[inline]
    pub fn left_en_passant(&self, position: Position) -> Position {
        (u8::from(position) - 1).into()
    }
    #[inline]
    pub fn right_en_passant(&self, position: Position) -> Position {
        (u8::from(position) + 1).into()
    }
}

fn add_move_if_valid(
    target: Position,
    moves: &mut Vec<Move>,
    center: Position,
    state: &GameState,
    color: PieceColor,
) -> bool {
    if let Some(piece) = state.board.get(target) {
        if piece.color() != color {
            moves.push(Move::Capture {
                from: center,
                to: target,
                captured: piece,
            });
        }
        true
    } else {
        moves.push(Move::Normal {
            from: center,
            to: target,
        });
        false
    }
}

fn make_bischop_moves(center: Position, state: &GameState, color: PieceColor) -> Vec<Move> {
    let mut moves = Vec::new();
    let center_flat = u8::from(center);

    let tl_steps = u8::min(center.rank(),     center.file());
    let tr_steps = u8::min(center.rank(),     7 - center.file());
    let bl_steps = u8::min(7 - center.rank(), center.file());
    let br_steps = u8::min(7 - center.rank(), 7 - center.file());

    for i in 1..=tl_steps {
        if add_move_if_valid(
            (center_flat - 9 * i).into(),
            &mut moves,
            center,
            state,
            color,
        ) {
            break;
        }
    }
    for i in 1..=tr_steps {
        if add_move_if_valid(
            (center_flat - 7 * i).into(),
            &mut moves,
            center,
            state,
            color,
        ) {
            break;
        }
    }
    for i in 1..=bl_steps {
        if add_move_if_valid(
            (center_flat + 7 * i).into(),
            &mut moves,
            center,
            state,
            color,
        ) {
            break;
        }
    }
    for i in 1..=br_steps {
        if add_move_if_valid(
            (center_flat + 9 * i).into(),
            &mut moves,
            center,
            state,
            color,
        ) {
            break;
        }
    }
    moves
}
fn make_rook_moves(center: Position, state: &GameState, color: PieceColor) -> Vec<Move> {
    let mut moves = Vec::new();

    for rank in center.rank() + 1..8 {
        if add_move_if_valid(
            Position::new(rank, center.file()),
            &mut moves,
            center,
            state,
            color,
        ) {
            break;
        }
    }
    for rank in (0..center.rank()).rev() {
        if add_move_if_valid(
            Position::new(rank, center.file()),
            &mut moves,
            center,
            state,
            color,
        ) {
            break;
        }
    }
    for file in center.file() + 1..8 {
        if add_move_if_valid(
            Position::new(center.rank(), file),
            &mut moves,
            center,
            state,
            color,
        ) {
            break;
        }
    }
    for file in (0..center.file()).rev() {
        if add_move_if_valid(
            Position::new(center.rank(), file),
            &mut moves,
            center,
            state,
            color,
        ) {
            break;
        }
    }
    moves
}

fn make_queen_moves(center: Position, state: &GameState, color: PieceColor) -> Vec<Move> {
    let mut moves = Vec::new();
    moves.append(&mut make_bischop_moves(center, state, color));
    moves.append(&mut make_rook_moves(center, state, color));
    moves
}

fn make_knight_moves(center: Position, state: &GameState, color: PieceColor) -> Vec<Move> {
    let mut moves = Vec::new();
    let center_flat = u8::from(center);

    if center.rank() > 1 {
        if center.file() > 0 {
            add_move_if_valid((center_flat - 17).into(), &mut moves, center, state, color);
        }
        if center.file() < 7 {
            add_move_if_valid((center_flat - 15).into(), &mut moves, center, state, color);
        }
    }
    if center.rank() > 0 {
        if center.file() > 1 {
            add_move_if_valid((center_flat - 10).into(), &mut moves, center, state, color);
        }
        if center.file() < 6 {
            add_move_if_valid((center_flat - 6).into(), &mut moves, center, state, color);
        }
    }
    if center.rank() < 6 {
        if center.file() > 0 {
            add_move_if_valid((center_flat + 15).into(), &mut moves, center, state, color);
        }
        if center.file() < 7 {
            add_move_if_valid((center_flat + 17).into(), &mut moves, center, state, color);
        }
    }
    if center.rank() < 7 {
        if center.file() > 1 {
            add_move_if_valid((center_flat + 6).into(), &mut moves, center, state, color);
        }
        if center.file() < 6 {
            add_move_if_valid((center_flat + 10).into(), &mut moves, center, state, color);
        }
    }

    moves
}

fn make_king_moves(center: Position, state: &GameState, color: PieceColor) -> Vec<Move> {
    let mut moves = Vec::new();
    let center_flat = u8::from(center);

    if center.file() > 0 {
        if center.rank() > 0 {
            add_move_if_valid((center_flat - 9).into(), &mut moves, center, state, color);
        }
        if center.rank() < 7 {
            add_move_if_valid((center_flat + 7).into(), &mut moves, center, state, color);
        }
        add_move_if_valid((center_flat - 1).into(), &mut moves, center, state, color);
    }
    if center.file() < 7 {
        if center.rank() > 0 {
            add_move_if_valid((center_flat - 7).into(), &mut moves, center, state, color);
        }
        if center.rank() < 7 {
            add_move_if_valid((center_flat + 9).into(), &mut moves, center, state, color);
        }
        add_move_if_valid((center_flat + 1).into(), &mut moves, center, state, color);
    }
    if center.rank() > 0 {
        add_move_if_valid((center_flat - 8).into(), &mut moves, center, state, color);
    }
    if center.rank() < 7 {
        add_move_if_valid((center_flat + 8).into(), &mut moves, center, state, color);
    }
    moves
}
fn make_pawn_promotions(from: Position, to: Position, color: PieceColor) -> Vec<Move> {
    vec![
        Move::Promotion {
            from,
            to,
            promoted: Piece::new(PieceKind::Queen, color),
        },
        Move::Promotion {
            from,
            to,
            promoted: Piece::new(PieceKind::Rook, color),
        },
        Move::Promotion {
            from,
            to,
            promoted: Piece::new(PieceKind::Bishop, color),
        },
        Move::Promotion {
            from,
            to,
            promoted: Piece::new(PieceKind::Knight, color),
        },
    ]
}
fn make_pawn_promotion_captures(
    from: Position,
    to: Position,
    captured: Piece,
    color: PieceColor,
) -> Vec<Move> {
    vec![
        Move::PromotionCapture {
            from,
            to,
            captured,
            promoted: Piece::new(PieceKind::Queen, color),
        },
        Move::PromotionCapture {
            from,
            to,
            captured,
            promoted: Piece::new(PieceKind::Rook, color),
        },
        Move::PromotionCapture {
            from,
            to,
            captured,
            promoted: Piece::new(PieceKind::Bishop, color),
        },
        Move::PromotionCapture {
            from,
            to,
            captured,
            promoted: Piece::new(PieceKind::Knight, color),
        },
    ]
}

fn make_pawn_pushes(center: Position, state: &GameState, info: &PawnMoveInfo) -> Vec<Move> {
    let mut moves = Vec::new();
    let push = info.push(center);

    if state.board.get(push).is_none() {
        if center.rank() == info.start_rank {
            let double_push = info.double_push(center);
            if state.board.get(double_push).is_none() {
                moves.push(Move::DoublePawnPush {
                    from: center,
                    to: double_push,
                    en_passant: push,
                });
            }
        }
        if push.rank() == info.promotion_rank {
            moves.append(&mut make_pawn_promotions(center, push, info.color));
        } else {
            moves.push(Move::Normal {
                from: center,
                to: push,
            });
        }
    }
    moves
}

fn make_pawn_captures(center: Position, state: &GameState, info: &PawnMoveInfo) -> Vec<Move> {
    let mut moves = Vec::new();
    if center.file() > 0 {
        let left_target = info.left_capture(center);
        if let Some(piece) = state.board.get(left_target) {
            if piece.color() != info.color {
                if left_target.rank() == info.promotion_rank {
                    moves.append(&mut make_pawn_promotion_captures(
                        center,
                        left_target,
                        piece,
                        info.color,
                    ));
                } else {
                    moves.push(Move::Capture {
                        from: center,
                        to: left_target,
                        captured: piece,
                    });
                }
            }
        } else if let Some(ep) = state.en_passant {
            if ep == left_target {
                moves.push(Move::EnPassant {
                    from: center,
                    to: left_target,
                    captured: info.left_en_passant(center),
                });
            }
        }
    }
    if center.file() < 7 {
        let right_target = info.right_capture(center);
        if let Some(piece) = state.board.get(right_target) {
            if piece.color() != info.color {
                if right_target.rank() == info.promotion_rank {
                    moves.append(&mut make_pawn_promotion_captures(
                        center,
                        right_target,
                        piece,
                        info.color,
                    ));
                } else {
                    moves.push(Move::Capture {
                        from: center,
                        to: right_target,
                        captured: piece,
                    });
                }
            }
        } else if let Some(ep) = state.en_passant {
            if ep == right_target {
                moves.push(Move::EnPassant {
                    from: center,
                    to: right_target,
                    captured: info.right_en_passant(center),
                });
            }
        }
    }
    moves
}
fn make_pawn_moves(center: Position, state: &GameState, info: &PawnMoveInfo) -> Vec<Move> {
    let mut moves = Vec::new();
    moves.append(&mut make_pawn_pushes(center, state, info));
    moves.append(&mut make_pawn_captures(center, state, info));
    moves
}

fn make_castle_moves(state: &GameState, color: PieceColor) -> Vec<Move> {
    let mut moves = Vec::new();
    let king = if color == PieceColor::White {
        Position::new(0, 4)
    } else {
        Position::new(7, 4)
    };

    if state.can_castle(color, CastleSide::KingSide) {
        let rook = Position::new(0, 7);
        let rook_target = Position::new(0, 5);
        let king_target = Position::new(0, 6);

        if state.board.get(rook_target).is_none() && state.board.get(king_target).is_none() {
            moves.push(Move::Castle {
                from: king,
                to: king_target,
                rook_from: rook,
                rook_to: rook_target,
            });
        }
    }
    if state.can_castle(state.turn, CastleSide::QueenSide) {
        let rook = Position::new(0, 0);
        let rook_target = Position::new(0, 3);
        let king_target = Position::new(0, 2);
        if state.board.get(rook_target).is_none()
            && state.board.get(king_target).is_none()
            && state.board.get(Position::new(0, 1)).is_none()
        {
            moves.push(Move::Castle {
                from: king,
                to: king_target,
                rook_from: rook,
                rook_to: rook_target,
            });
        }
    }
    moves
}
pub fn get_moves_for_square(
    square: Square,
    position: Position,
    state: &GameState,
    color: PieceColor,
) -> Vec<Move> {
    if square.is_none() {
        return Vec::new();
    }
    let piece = square.unwrap();
    if piece.color() != color {
        return Vec::new();
    }
    match piece.kind() {
        PieceKind::Pawn => make_pawn_moves(position, state, &PawnMoveInfo::new(color)),
        PieceKind::Knight => make_knight_moves(position, state, color),
        PieceKind::Bishop => make_bischop_moves(position, state, color),
        PieceKind::Rook => make_rook_moves(position, state, color),
        PieceKind::Queen => make_queen_moves(position, state, color),
        PieceKind::King => make_king_moves(position, state, color),
    }
}
pub fn get_moves(state: &GameState, color: PieceColor) -> Vec<Move> {
    let mut moves = Vec::new();
    for (index, square) in state.board.squares().iter().enumerate() {
        let position = Position::from(index);
        moves.append(&mut get_moves_for_square(*square, position, state, color));
    }
    moves.append(&mut make_castle_moves(state, color));
    moves
}

pub fn prune_moves_into_check(mut moves: Vec<Move>, state: &GameState) -> Vec<Move> {
    let mut i = 0;
    while i < moves.len() {
        if let Move::Castle { rook_to, .. } = moves[i] {
            if state.is_in_check() || get_moves(state, state.turn.opposite())
                                        .iter()
                                        .any(|m| m.to() == rook_to)
            {
                moves.remove(i);
                continue;
            } else {
                i += 1;
            }
        }
        let mut new_state = (*state).clone();
        new_state.apply_move(moves[i]);
        if new_state.is_in_check() {
            moves.remove(i);
        } else {
            i += 1;
        }
    }
    moves
}

impl Move {
    #[cfg(not(tarpaulin_include))]
    pub fn from(&self) -> Position {
        match self {
            Move::Normal { from, .. } => *from,
            Move::Capture { from, .. } => *from,
            Move::EnPassant { from, .. } => *from,
            Move::DoublePawnPush { from, .. } => *from,
            Move::Promotion { from, .. } => *from,
            Move::PromotionCapture { from, .. } => *from,
            Move::Castle { from, .. } => *from,
        }
    }
    #[cfg(not(tarpaulin_include))]
    pub fn to(&self) -> Position {
        match self {
            Move::Normal { to, .. } => *to,
            Move::Capture { to, .. } => *to,
            Move::EnPassant { to, .. } => *to,
            Move::DoublePawnPush { to, .. } => *to,
            Move::Promotion { to, .. } => *to,
            Move::PromotionCapture { to, .. } => *to,
            Move::Castle { to, .. } => *to,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Board;
    use crate::piece::Piece;
    use crate::position::Position;
    use crate::state::GameState;
    use test_case::test_case;
    use devutil::VecAsserts;

    #[test]
    fn test_get_moves_pawn_pushes() {
        let mut board = Board::empty();
        board.set(
            Position::new(1, 0),
            Some(Piece::new(PieceKind::Pawn, PieceColor::White)),
        );
        let state = GameState::new(board, PieceColor::White, 0b0000, None, 0, 0);
        let moves = get_moves(&state, PieceColor::White);
        moves.assert_len(2);
        assert!(moves.contains(&Move::Normal {
            from: Position::new(1, 0),
            to: Position::new(2, 0)
        }));
        assert!(moves.contains(&Move::DoublePawnPush {
            from: Position::new(1, 0),
            to: Position::new(3, 0),
            en_passant: Position::new(2, 0)
        }));

        let mut board = Board::empty();
        board.set(
            Position::new(6, 0),
            Some(Piece::new(PieceKind::Pawn, PieceColor::Black)),
        );
        let state = GameState::new(board, PieceColor::Black, 0b0000, None, 0, 0);
        let moves = get_moves(&state, PieceColor::Black);
        assert_eq!(moves.len(), 2);
        assert!(moves.contains(&Move::Normal {
            from: Position::new(6, 0),
            to: Position::new(5, 0)
        }));
        assert!(moves.contains(&Move::DoublePawnPush {
            from: Position::new(6, 0),
            to: Position::new(4, 0),
            en_passant: Position::new(5, 0)
        }));
    }
    #[test]
    fn test_get_moves_pawn_captures() {
        let mut board = Board::empty();
        board.set(
            Position::new(4, 2),
            Some(Piece::new(PieceKind::Pawn, PieceColor::White)),
        );
        board.set(
            Position::new(5, 1),
            Some(Piece::new(PieceKind::Pawn, PieceColor::Black)),
        );
        board.set(
            Position::new(5, 3),
            Some(Piece::new(PieceKind::Pawn, PieceColor::Black)),
        );

        let state = GameState::new(board, PieceColor::White, 0b0000, None, 0, 0);
        let moves = get_moves(&state, PieceColor::White);
        moves.assert_len(3);
        assert!(moves.contains(&Move::Capture {
            from: Position::new(4, 2),
            to: Position::new(5, 1),
            captured: Piece::new(PieceKind::Pawn, PieceColor::Black)
        }));
        assert!(moves.contains(&Move::Capture {
            from: Position::new(4, 2),
            to: Position::new(5, 3),
            captured: Piece::new(PieceKind::Pawn, PieceColor::Black)
        }));

        let mut board = Board::empty();
        board.set(
            Position::new(4, 2),
            Some(Piece::new(PieceKind::Pawn, PieceColor::Black)),
        );
        board.set(
            Position::new(3, 1),
            Some(Piece::new(PieceKind::Pawn, PieceColor::White)),
        );
        board.set(
            Position::new(3, 3),
            Some(Piece::new(PieceKind::Pawn, PieceColor::White)),
        );

        let state = GameState::new(board, PieceColor::Black, 0b0000, None, 0, 0);
        let moves = get_moves(&state, PieceColor::Black);
        moves.assert_len(3);
        assert!(moves.contains(&Move::Capture {
            from: Position::new(4, 2),
            to: Position::new(3, 1),
            captured: Piece::new(PieceKind::Pawn, PieceColor::White)
        }));
        assert!(moves.contains(&Move::Capture {
            from: Position::new(4, 2),
            to: Position::new(3, 3),
            captured: Piece::new(PieceKind::Pawn, PieceColor::White)
        }));
    }
    #[test]
    fn test_get_moves_pawn_en_passant() {
        // White left
        let mut board = Board::empty();
        board.set(
            Position::new(4, 2),
            Some(Piece::new(PieceKind::Pawn, PieceColor::White)),
        );
        board.set(
            Position::new(4, 1),
            Some(Piece::new(PieceKind::Pawn, PieceColor::Black)),
        );

        let state = GameState::new(
            board,
            PieceColor::White,
            0b0000,
            Some(Position::new(5, 1)),
            0,
            0,
        );
        let moves = get_moves(&state, PieceColor::White);
        moves.assert_len(2);
        assert!(moves.contains(&Move::Normal {
            from: Position::new(4, 2),
            to: Position::new(5, 2)
        }));
        assert!(moves.contains(&Move::EnPassant {
            from: Position::new(4, 2),
            to: Position::new(5, 1),
            captured: Position::new(4, 1)
        }));

        // White right
        let mut board = Board::empty();
        board.set(
            Position::new(4, 2),
            Some(Piece::new(PieceKind::Pawn, PieceColor::White)),
        );
        board.set(
            Position::new(4, 3),
            Some(Piece::new(PieceKind::Pawn, PieceColor::Black)),
        );

        let state = GameState::new(
            board,
            PieceColor::White,
            0b0000,
            Some(Position::new(5, 3)),
            0,
            0,
        );
        let moves = get_moves(&state, PieceColor::White);
        moves.assert_len(2);
        assert!(moves.contains(&Move::Normal {
            from: Position::new(4, 2),
            to: Position::new(5, 2)
        }));
        assert!(moves.contains(&Move::EnPassant {
            from: Position::new(4, 2),
            to: Position::new(5, 3),
            captured: Position::new(4, 3)
        }));

        // Black left
        let mut board = Board::empty();
        board.set(
            Position::new(4, 2),
            Some(Piece::new(PieceKind::Pawn, PieceColor::Black)),
        );
        board.set(
            Position::new(4, 1),
            Some(Piece::new(PieceKind::Pawn, PieceColor::White)),
        );

        let state = GameState::new(
            board,
            PieceColor::Black,
            0b0000,
            Some(Position::new(3, 1)),
            0,
            0,
        );
        let moves = get_moves(&state, PieceColor::Black);
        moves.assert_len(2);
        assert!(moves.contains(&Move::Normal {
            from: Position::new(4, 2),
            to: Position::new(3, 2)
        }));
        assert!(moves.contains(&Move::EnPassant {
            from: Position::new(4, 2),
            to: Position::new(3, 1),
            captured: Position::new(4, 1)
        }));

        // Black right
        let mut board = Board::empty();
        board.set(
            Position::new(4, 2),
            Some(Piece::new(PieceKind::Pawn, PieceColor::Black)),
        );
        board.set(
            Position::new(4, 3),
            Some(Piece::new(PieceKind::Pawn, PieceColor::White)),
        );

        let state = GameState::new(
            board,
            PieceColor::Black,
            0b0000,
            Some(Position::new(3, 3)),
            0,
            0,
        );
        let moves = get_moves(&state, PieceColor::Black);
        moves.assert_len(2);
        assert!(moves.contains(&Move::Normal {
            from: Position::new(4, 2),
            to: Position::new(3, 2)
        }));
        assert!(moves.contains(&Move::EnPassant {
            from: Position::new(4, 2),
            to: Position::new(3, 3),
            captured: Position::new(4, 3)
        }));
    }

    #[test]
    fn test_get_moves_pawn_promotions() {
        let mut board = Board::empty();
        board.set(
            Position::new(6, 0),
            Some(Piece::new(PieceKind::Pawn, PieceColor::White)),
        );
        let state = GameState::new(board, PieceColor::White, 0b0000, None, 0, 0);
        let moves = get_moves(&state, PieceColor::White);
        moves.assert_len(4);
        assert!(moves.contains(&Move::Promotion {
            from: Position::new(6, 0),
            to: Position::new(7, 0),
            promoted: Piece::new(PieceKind::Queen, PieceColor::White)
        }));
        assert!(moves.contains(&Move::Promotion {
            from: Position::new(6, 0),
            to: Position::new(7, 0),
            promoted: Piece::new(PieceKind::Rook, PieceColor::White)
        }));
        assert!(moves.contains(&Move::Promotion {
            from: Position::new(6, 0),
            to: Position::new(7, 0),
            promoted: Piece::new(PieceKind::Bishop, PieceColor::White)
        }));
        assert!(moves.contains(&Move::Promotion {
            from: Position::new(6, 0),
            to: Position::new(7, 0),
            promoted: Piece::new(PieceKind::Knight, PieceColor::White)
        }));
    }

    #[test]
    fn test_get_moves_pawn_promotion_captures() {
        let mut board = Board::empty();
        board.set(
            Position::new(6, 1),
            Some(Piece::new(PieceKind::Pawn, PieceColor::White)),
        );
        board.set(
            Position::new(7, 0),
            Some(Piece::new(PieceKind::Bishop, PieceColor::Black)),
        );
        board.set(
            Position::new(7, 2),
            Some(Piece::new(PieceKind::Rook, PieceColor::Black)),
        );
        let state = GameState::new(board, PieceColor::White, 0b0000, None, 0, 0);
        let moves = get_moves(&state, PieceColor::White);
        moves.assert_len(12);

        assert!(moves.contains(&Move::Promotion {
            from: Position::new(6, 1),
            to: Position::new(7, 1),
            promoted: Piece::new(PieceKind::Queen, PieceColor::White)
        }));
        assert!(moves.contains(&Move::Promotion {
            from: Position::new(6, 1),
            to: Position::new(7, 1),
            promoted: Piece::new(PieceKind::Rook, PieceColor::White)
        }));
        assert!(moves.contains(&Move::Promotion {
            from: Position::new(6, 1),
            to: Position::new(7, 1),
            promoted: Piece::new(PieceKind::Bishop, PieceColor::White)
        }));
        assert!(moves.contains(&Move::Promotion {
            from: Position::new(6, 1),
            to: Position::new(7, 1),
            promoted: Piece::new(PieceKind::Knight, PieceColor::White)
        }));

        assert!(moves.contains(&Move::PromotionCapture {
            from: Position::new(6, 1),
            to: Position::new(7, 0),
            captured: Piece::new(PieceKind::Bishop, PieceColor::Black),
            promoted: Piece::new(PieceKind::Queen, PieceColor::White)
        }));
        assert!(moves.contains(&Move::PromotionCapture {
            from: Position::new(6, 1),
            to: Position::new(7, 0),
            captured: Piece::new(PieceKind::Bishop, PieceColor::Black),
            promoted: Piece::new(PieceKind::Rook, PieceColor::White)
        }));
        assert!(moves.contains(&Move::PromotionCapture {
            from: Position::new(6, 1),
            to: Position::new(7, 0),
            captured: Piece::new(PieceKind::Bishop, PieceColor::Black),
            promoted: Piece::new(PieceKind::Bishop, PieceColor::White)
        }));
        assert!(moves.contains(&Move::PromotionCapture {
            from: Position::new(6, 1),
            to: Position::new(7, 0),
            captured: Piece::new(PieceKind::Bishop, PieceColor::Black),
            promoted: Piece::new(PieceKind::Knight, PieceColor::White)
        }));

        assert!(moves.contains(&Move::PromotionCapture {
            from: Position::new(6, 1),
            to: Position::new(7, 2),
            captured: Piece::new(PieceKind::Rook, PieceColor::Black),
            promoted: Piece::new(PieceKind::Queen, PieceColor::White)
        }));
        assert!(moves.contains(&Move::PromotionCapture {
            from: Position::new(6, 1),
            to: Position::new(7, 2),
            captured: Piece::new(PieceKind::Rook, PieceColor::Black),
            promoted: Piece::new(PieceKind::Rook, PieceColor::White)
        }));
        assert!(moves.contains(&Move::PromotionCapture {
            from: Position::new(6, 1),
            to: Position::new(7, 2),
            captured: Piece::new(PieceKind::Rook, PieceColor::Black),
            promoted: Piece::new(PieceKind::Bishop, PieceColor::White)
        }));
        assert!(moves.contains(&Move::PromotionCapture {
            from: Position::new(6, 1),
            to: Position::new(7, 2),
            captured: Piece::new(PieceKind::Rook, PieceColor::Black),
            promoted: Piece::new(PieceKind::Knight, PieceColor::White)
        }));
    }

    #[test]
    fn test_get_moves_pawn_edge_captures() {
        let mut board = Board::empty();
        board.set(
            Position::new(4, 0),
            Some(Piece::new(PieceKind::Pawn, PieceColor::White)),
        );
        board.set(
            Position::new(5, 1),
            Some(Piece::new(PieceKind::Pawn, PieceColor::Black)),
        );

        let state = GameState::new(board, PieceColor::White, 0b0000, None, 0, 0);
        let moves = get_moves(&state, PieceColor::White);
        moves.assert_len(2);
        assert!(moves.contains(&Move::Normal {
            from: Position::new(4, 0),
            to: Position::new(5, 0)
        }));
        assert!(moves.contains(&Move::Capture {
            from: Position::new(4, 0),
            to: Position::new(5, 1),
            captured: Piece::new(PieceKind::Pawn, PieceColor::Black)
        }));

        let mut board = Board::empty();
        board.set(
            Position::new(4, 7),
            Some(Piece::new(PieceKind::Pawn, PieceColor::White)),
        );
        board.set(
            Position::new(5, 6),
            Some(Piece::new(PieceKind::Pawn, PieceColor::Black)),
        );

        let state = GameState::new(board, PieceColor::White, 0b0000, None, 0, 0);
        let moves = get_moves(&state, PieceColor::White);
        moves.assert_len(2);
        assert!(moves.contains(&Move::Normal {
            from: Position::new(4, 7),
            to: Position::new(5, 7)
        }));
        assert!(moves.contains(&Move::Capture {
            from: Position::new(4, 7),
            to: Position::new(5, 6),
            captured: Piece::new(PieceKind::Pawn, PieceColor::Black)
        }));
    }

    #[test]
    fn test_get_moves_pawn_start_rank_captures() {
        let mut board = Board::empty();
        board.set(
            Position::new(1, 1),
            Some(Piece::new(PieceKind::Pawn, PieceColor::White)),
        );
        board.set(
            Position::new(2, 0),
            Some(Piece::new(PieceKind::Pawn, PieceColor::Black)),
        );
        board.set(
            Position::new(2, 2),
            Some(Piece::new(PieceKind::Pawn, PieceColor::Black)),
        );

        let state = GameState::new(board, PieceColor::White, 0b0000, None, 0, 0);
        let moves = get_moves(&state, PieceColor::White);
        moves.assert_len(4);
        assert!(moves.contains(&Move::Normal {
            from: Position::new(1, 1),
            to: Position::new(2, 1)
        }));
        assert!(moves.contains(&Move::DoublePawnPush {
            from: Position::new(1, 1),
            to: Position::new(3, 1),
            en_passant: Position::new(2, 1)
        }));
        assert!(moves.contains(&Move::Capture {
            from: Position::new(1, 1),
            to: Position::new(2, 0),
            captured: Piece::new(PieceKind::Pawn, PieceColor::Black)
        }));
        assert!(moves.contains(&Move::Capture {
            from: Position::new(1, 1),
            to: Position::new(2, 2),
            captured: Piece::new(PieceKind::Pawn, PieceColor::Black)
        }));
    } 

    #[test_case(Position::new(0, 0), 7)]
    #[test_case(Position::new(1, 1), 9)]
    #[test_case(Position::new(2, 2), 11)]
    #[test_case(Position::new(3, 3), 13)]
    fn test_get_moves_bishop_layout(position: Position, expected_count: usize) {        
        let mut board = Board::empty();
        board.set(
            position,
            Some(Piece::new(PieceKind::Bishop, PieceColor::White)),
        );
        let state = GameState::new(board, PieceColor::White, 0b0000, None, 0, 0);
        let moves = get_moves(&state, PieceColor::White);   
        assert_eq!(moves.len(), expected_count);

        moves.assert_forall(|m| m.from() == position);   
        moves.assert_forall(|m| m.to() != position);     
        moves.assert_forall(|m| 
            u8::abs_diff(m.from().rank(), m.to().rank()) 
            == u8::abs_diff(m.from().file(), m.to().file())
        );
    }    

    #[test]
    fn test_get_moves_bishop_captures() {
        let mut board = Board::empty();
        
        board.set(
            Position::new(4, 4),
            Some(Piece::new(PieceKind::Bishop, PieceColor::White)),
        );
        board.set(
            Position::new(5, 5),
            Some(Piece::new(PieceKind::Pawn, PieceColor::Black)),
        );
        board.set(
            Position::new(3, 5),
            Some(Piece::new(PieceKind::Pawn, PieceColor::Black)),
        );
        board.set(
            Position::new(5, 3),
            Some(Piece::new(PieceKind::Pawn, PieceColor::Black)),
        );
        board.set(
            Position::new(3, 3),
            Some(Piece::new(PieceKind::Pawn, PieceColor::Black)),
        );

        let state = GameState::new(board, PieceColor::White, 0b0000, None, 0, 0);
        let moves = get_moves(&state, PieceColor::White);
        assert_eq!(moves.len(), 4);
        moves.assert_forall(|m| matches!(m, Move::Capture{..}));
        moves.assert_forall(
            |m| u8::abs_diff(m.from().rank(), m.to().rank()) == 1
        );
        moves.assert_forall(
            |m| u8::abs_diff(m.from().file(), m.to().file()) == 1
        );
    }

    #[test]    
    fn test_get_moves_bishop_blocked() {
        let mut board = Board::empty();
        
        board.set(
            Position::new(4, 4),
            Some(Piece::new(PieceKind::Bishop, PieceColor::White)),
        );
        board.set(
            Position::new(5, 5),
            Some(Piece::new(PieceKind::Pawn, PieceColor::White)),
        );
        board.set(
            Position::new(3, 5),
            Some(Piece::new(PieceKind::Pawn, PieceColor::White)),
        );
        board.set(
            Position::new(5, 3),
            Some(Piece::new(PieceKind::Pawn, PieceColor::White)),
        );
        board.set(
            Position::new(3, 3),
            Some(Piece::new(PieceKind::Pawn, PieceColor::White)),
        );

        let state = GameState::new(board, PieceColor::White, 0b0000, None, 0, 0);
        let moves = get_moves_for_square(
            board.get(Position::new(4, 4)),
            Position::new(4, 4),
            &state,
            PieceColor::White
        );
        moves.assert_empty();
    }

    #[test_case(Position::new(0, 0))]
    #[test_case(Position::new(4, 4))]
    #[test_case(Position::new(7, 7))]
    fn test_get_moves_rook_layout(position: Position) {        
        let mut board = Board::empty();
        board.set(
            position,
            Some(Piece::new(PieceKind::Rook, PieceColor::White)),
        );

        let state = GameState::new(board, PieceColor::White, 0b0000, None, 0, 0);
        let moves = get_moves(&state, PieceColor::White);   
        
        moves.assert_len(14);
        moves.assert_forall(|m| m.from() == position);
        moves.assert_forall(|m| m.to() != position);
        moves.assert_forall(|m| 
            m.from().rank() == m.to().rank() || m.from().file() == m.to().file()
        );
    }

    #[test]
    fn test_get_moves_rook_captures() {
        let mut board = Board::empty();
        board.set(
            Position::new(4, 4),
            Some(Piece::new(PieceKind::Rook, PieceColor::White)),
        );
        board.set(
            Position::new(5, 4),
            Some(Piece::new(PieceKind::Pawn, PieceColor::Black)),
        );
        board.set(
            Position::new(3, 4),
            Some(Piece::new(PieceKind::Pawn, PieceColor::Black)),
        );
        board.set(
            Position::new(4, 5),
            Some(Piece::new(PieceKind::Pawn, PieceColor::Black)),
        );
        board.set(
            Position::new(4, 3),
            Some(Piece::new(PieceKind::Pawn, PieceColor::Black)),
        );

        let state = GameState::new(board, PieceColor::White, 0b0000, None, 0, 0);
        let moves = get_moves(&state, PieceColor::White);
        moves.assert_len(4);
        moves.assert_forall(|m| matches!(m, Move::Capture{..}));        
        moves.assert_forall(
            |m| m.from().rank() == m.to().rank() || m.from().file() == m.to().file()
        );
    }

    #[test]
    fn test_get_moves_rook_blocked() {
        let mut board = Board::empty();
        board.set(
            Position::new(4, 4),
            Some(Piece::new(PieceKind::Rook, PieceColor::White)),
        );
        board.set(
            Position::new(5, 4),
            Some(Piece::new(PieceKind::Pawn, PieceColor::White)),
        );
        board.set(
            Position::new(3, 4),
            Some(Piece::new(PieceKind::Pawn, PieceColor::White)),
        );
        board.set(
            Position::new(4, 5),
            Some(Piece::new(PieceKind::Pawn, PieceColor::White)),
        );
        board.set(
            Position::new(4, 3),
            Some(Piece::new(PieceKind::Pawn, PieceColor::White)),
        );

        let state = GameState::new(board, PieceColor::White, 0b0000, None, 0, 0);
        let moves = get_moves_for_square(
            board.get(Position::new(4, 4)),
            Position::new(4, 4),
            &state,
            PieceColor::White
        );
        moves.assert_empty();
    }

    #[test_case(Position::new(0, 0), 2)]
    #[test_case(Position::new(0, 7), 2)]
    #[test_case(Position::new(7, 0), 2)]
    #[test_case(Position::new(7, 7), 2)]
    #[test_case(Position::new(0, 1), 3)]
    #[test_case(Position::new(1, 1), 4)]
    #[test_case(Position::new(0, 2), 4)]
    #[test_case(Position::new(1, 2), 6)]
    #[test_case(Position::new(2, 2), 8)]    

    fn test_get_moves_knight_layout(position: Position, expected_count: usize) {        
        let mut board = Board::empty();
        board.set(
            position,
            Some(Piece::new(PieceKind::Knight, PieceColor::White)),
        );

        let state = GameState::new(board, PieceColor::White, 0b0000, None, 0, 0);
        let moves = get_moves(&state, PieceColor::White);   
        
        moves.assert_len(expected_count);
        moves.assert_forall(|m| m.from() == position);
        moves.assert_forall(|m| m.to() != position);
        moves.assert_forall(|m| 
            (u8::abs_diff(m.from().rank(), m.to().rank()) == 2 
                && u8::abs_diff(m.from().file(), m.to().file()) == 1) 
            || (u8::abs_diff(m.from().rank(), m.to().rank()) == 1 
                && u8::abs_diff(m.from().file(), m.to().file()) == 2)
        );
    }
    
    #[test_case(Position::new(0, 0), 3)]
    #[test_case(Position::new(0, 7), 3)]
    #[test_case(Position::new(7, 0), 3)]
    #[test_case(Position::new(7, 7), 3)]
    #[test_case(Position::new(0, 3), 5)]
    #[test_case(Position::new(3, 3), 8)]
    fn test_get_moves_king_layout(position: Position, expected_count: usize) {        
        let mut board = Board::empty();
        board.set(
            position,
            Some(Piece::new(PieceKind::King, PieceColor::White)),
        );

        let state = GameState::new(board, PieceColor::White, 0b0000, None, 0, 0);
        let moves = get_moves(&state, PieceColor::White);   
        
        moves.assert_len(expected_count);
        moves.assert_forall(|m| m.from() == position);
        moves.assert_forall(|m| m.to() != position);
        moves.assert_forall(|m| 
            u8::abs_diff(m.from().rank(), m.to().rank()) <= 1 
            && u8::abs_diff(m.from().file(), m.to().file()) <= 1
        );
    }

    #[test]
    fn test_get_moves_king_captures() {
        let mut board = Board::empty();
        board.set(
            Position::new(4, 4),
            Some(Piece::new(PieceKind::King, PieceColor::White)),
        );
        board.set(
            Position::new(3, 5),
            Some(Piece::new(PieceKind::Pawn, PieceColor::Black)),
        );
        board.set(
            Position::new(4, 5),
            Some(Piece::new(PieceKind::Pawn, PieceColor::Black)),
        );
        board.set(
            Position::new(5, 5),
            Some(Piece::new(PieceKind::Pawn, PieceColor::Black)),
        );
        board.set(
            Position::new(5, 4),
            Some(Piece::new(PieceKind::Pawn, PieceColor::Black)),
        );
        board.set(
            Position::new(5, 3),
            Some(Piece::new(PieceKind::Pawn, PieceColor::Black)),
        );
        board.set(
            Position::new(4, 3),
            Some(Piece::new(PieceKind::Pawn, PieceColor::Black)),
        );
        board.set(
            Position::new(3, 3),
            Some(Piece::new(PieceKind::Pawn, PieceColor::Black)),
        );
        board.set(
            Position::new(3, 4),
            Some(Piece::new(PieceKind::Pawn, PieceColor::Black)),
        );

        let state = GameState::new(board, PieceColor::White, 0b0000, None, 0, 0);

        let moves = get_moves(&state, PieceColor::White);
        moves.assert_len(8);
        moves.assert_forall(|m| matches!(m, Move::Capture{..}));
        moves.assert_forall(|m| 
            u8::abs_diff(m.from().rank(), m.to().rank()) <= 1 
            && u8::abs_diff(m.from().file(), m.to().file()) <= 1
        );
    }

    #[test]
    fn test_get_moves_king_blocked() {
        let mut board = Board::empty();
        board.set(
            Position::new(4, 4),
            Some(Piece::new(PieceKind::King, PieceColor::White)),
        );
        board.set(
            Position::new(3, 5),
            Some(Piece::new(PieceKind::Pawn, PieceColor::White)),
        );
        board.set(
            Position::new(4, 5),
            Some(Piece::new(PieceKind::Pawn, PieceColor::White)),
        );
        board.set(
            Position::new(5, 5),
            Some(Piece::new(PieceKind::Pawn, PieceColor::White)),
        );
        board.set(
            Position::new(5, 4),
            Some(Piece::new(PieceKind::Pawn, PieceColor::White)),
        );
        board.set(
            Position::new(5, 3),
            Some(Piece::new(PieceKind::Pawn, PieceColor::White)),
        );
        board.set(
            Position::new(4, 3),
            Some(Piece::new(PieceKind::Pawn, PieceColor::White)),
        );
        board.set(
            Position::new(3, 3),
            Some(Piece::new(PieceKind::Pawn, PieceColor::White)),
        );
        board.set(
            Position::new(3, 4),
            Some(Piece::new(PieceKind::Pawn, PieceColor::White)),
        );

        let state = GameState::new(board, PieceColor::White, 0b0000, None, 0, 0);

        let moves = get_moves_for_square(
            board.get(Position::new(4, 4)),
            Position::new(4, 4),
            &state,
            PieceColor::White
        );
        moves.assert_empty();
    }

    #[test]
    fn test_get_moves_castle_moves() {
        let mut board = Board::empty();
        board.set(
            Position::new(0, 4),
            Some(Piece::new(PieceKind::King, PieceColor::White)),
        );
        board.set(
            Position::new(0, 0),
            Some(Piece::new(PieceKind::Rook, PieceColor::White)),
        );
        board.set(
            Position::new(0, 7),
            Some(Piece::new(PieceKind::Rook, PieceColor::White)),
        );

        let state = GameState::new(board, PieceColor::White, 0b1111, None, 0, 0);

        let moves = get_moves(&state, PieceColor::White);
        let castle_moves = moves.into_iter()
                            .filter(|m| matches!(m, Move::Castle{..}))
                            .collect::<Vec<_>>();
        
        castle_moves.assert_len(2);
        assert!(castle_moves.contains(&Move::Castle {
            from: Position::new(0, 4),
            to: Position::new(0, 2),
            rook_from: Position::new(0, 0),
            rook_to: Position::new(0, 3)
        }));
        assert!(castle_moves.contains(&Move::Castle {
            from: Position::new(0, 4),
            to: Position::new(0, 6),
            rook_from: Position::new(0, 7),
            rook_to: Position::new(0, 5)
        }));
    }

    #[test]
    fn test_get_moves_castle_moves_require_rights() {
        let mut board = Board::empty();
        board.set(
            Position::new(0, 4),
            Some(Piece::new(PieceKind::King, PieceColor::White)),
        );
        board.set(
            Position::new(0, 0),
            Some(Piece::new(PieceKind::Rook, PieceColor::White)),
        );
        board.set(
            Position::new(0, 7),
            Some(Piece::new(PieceKind::Rook, PieceColor::White)),
        );

        let state = GameState::new(board, PieceColor::White, 0b0000, None, 0, 0);

        let moves = get_moves(&state, PieceColor::White);
        let castle_moves = moves.into_iter()
                            .filter(|m| matches!(m, Move::Castle{..}))
                            .collect::<Vec<_>>();
        
        castle_moves.assert_empty();
    }

    #[test]
    fn test_get_moves_castle_moves_blocked() {
        let mut board = Board::empty();
        board.set(
            Position::new(0, 4),
            Some(Piece::new(PieceKind::King, PieceColor::White)),
        );
        board.set(
            Position::new(0, 0),
            Some(Piece::new(PieceKind::Rook, PieceColor::White)),
        );
        board.set(
            Position::new(0, 7),
            Some(Piece::new(PieceKind::Rook, PieceColor::White)),
        );
        board.set(
            Position::new(0, 1),
            Some(Piece::new(PieceKind::Pawn, PieceColor::White)),
        );
        board.set(
            Position::new(0, 5),
            Some(Piece::new(PieceKind::Pawn, PieceColor::White)),
        );
        
        let state = GameState::new(board, PieceColor::White, 0b1111, None, 0, 0);
        
        let moves = get_moves(&state, PieceColor::White);
        let castle_moves = moves.into_iter()
                            .filter(|m| matches!(m, Move::Castle{..}))
                            .collect::<Vec<_>>();
        
        castle_moves.assert_empty();
    }

    #[test]
    fn test_prune_moves_into_check_force_escape() {
        let mut board = Board::empty();
        board.set(
            Position::new(4, 4),
            Some(Piece::new(PieceKind::King, PieceColor::White)),
        );
        board.set(
            Position::new(3, 3),
            Some(Piece::new(PieceKind::Pawn, PieceColor::Black)),
        );
        board.set(
            Position::new(3, 4),
            Some(Piece::new(PieceKind::Pawn, PieceColor::Black)),
        );
        board.set(
            Position::new(3, 5),
            Some(Piece::new(PieceKind::Pawn, PieceColor::Black)),
        );
        board.set(
            Position::new(4, 5),
            Some(Piece::new(PieceKind::Pawn, PieceColor::Black)),
        );
        board.set(
            Position::new(5, 5),
            Some(Piece::new(PieceKind::Pawn, PieceColor::Black)),
        );
        board.set(
            Position::new(5, 4),
            Some(Piece::new(PieceKind::Pawn, PieceColor::Black)),
        );
        board.set(
            Position::new(5, 3),
            Some(Piece::new(PieceKind::Pawn, PieceColor::Black)),
        );
        board.set(
            Position::new(4, 3),
            Some(Piece::new(PieceKind::Pawn, PieceColor::Black)),
        );
        board.set(
            Position::new(1, 1),
            Some(Piece::new(PieceKind::Pawn, PieceColor::White)),
        );

        let state = GameState::new(board, PieceColor::White, 0b0000, None, 0, 0);

        let moves = get_moves(&state, PieceColor::White);
        let moves = prune_moves_into_check(moves, &state);
        moves.assert_len(5);
        moves.assert_forall(|m| matches!(m, Move::Capture{ .. }));
        moves.assert_forall(|m| m.from() == Position::new(4, 4));        
    }
    
    #[test]
    fn test_prune_moves_into_check_force_block() {
        let mut board = Board::empty();
        board.set(
            Position::new(0, 0),
            Some(Piece::new(PieceKind::King, PieceColor::White)),
        );
        board.set(
            Position::new(0, 1),
            Some(Piece::new(PieceKind::Pawn, PieceColor::White)),
        );
        board.set(
            Position::new(1, 0),
            Some(Piece::new(PieceKind::Pawn, PieceColor::White)),
        );
        board.set(
            Position::new(4, 4),
            Some(Piece::new(PieceKind::Bishop, PieceColor::Black)),
        );

        let state = GameState::new(board, PieceColor::White, 0b0000, None, 0, 0);
        
        let moves = get_moves(&state, PieceColor::White);
        let moves = prune_moves_into_check(moves, &state);

        moves.assert_len(1);
        moves.assert_forall(|m| matches!(m, Move::Normal{..}));
        moves.assert_forall(|m| m.from() == Position::new(0, 1));
        moves.assert_forall(|m| m.to() == Position::new(1, 1));
    }

    #[test]
    fn test_prune_moves_into_check_prevent_pin_move() {
        let mut board = Board::empty();
        board.set(
            Position::new(0, 0),
            Some(Piece::new(PieceKind::King, PieceColor::White)),
        );
        board.set(
            Position::new(0, 1),
            Some(Piece::new(PieceKind::Pawn, PieceColor::White)),
        );
        board.set(
            Position::new(1, 1),
            Some(Piece::new(PieceKind::Pawn, PieceColor::White)),
        );
        board.set(
            Position::new(4, 4),
            Some(Piece::new(PieceKind::Bishop, PieceColor::Black)),
        );

        let state = GameState::new(board, PieceColor::White, 0b0000, None, 0, 0);
        
        let moves = get_moves(&state, PieceColor::White);
        let moves = prune_moves_into_check(moves, &state);

        moves.assert_len(1);
        moves.assert_forall(|m| matches!(m, Move::Normal{..}));
        moves.assert_forall(|m| m.from() == Position::new(0, 0));
        moves.assert_forall(|m| m.to() == Position::new(1, 0));
    }

    #[test]
    fn test_prune_moves_into_check_prevent_en_passant_exposing_capture() {
        let mut board = Board::empty();
        board.set(
            Position::new(4, 0),
            Some(Piece::new(PieceKind::King, PieceColor::White)),
        );
        board.set(
            Position::new(4, 1),
            Some(Piece::new(PieceKind::Pawn, PieceColor::White)),
        );
        board.set(
            Position::new(4, 2),
            Some(Piece::new(PieceKind::Pawn, PieceColor::Black)),
        );
        board.set(
            Position::new(4, 7),
            Some(Piece::new(PieceKind::Rook, PieceColor::Black)),
        );

        let state = GameState::new(board, PieceColor::White, 0b0000, Some(Position::new(5, 2)), 0, 0);
        
        let moves = get_moves(&state, PieceColor::White);
        let moves = prune_moves_into_check(moves, &state);
        println!("{:?}", moves);        
        moves.assert_len(4);
        moves.assert_forall(|m| !matches!(m, Move::EnPassant{..}));
    }

    #[test]
    fn test_prune_moves_into_check_prevent_king_into_check() {
        let mut board = Board::empty();
        board.set(
            Position::new(4, 1),
            Some(Piece::new(PieceKind::King, PieceColor::White)),
        );
        board.set(
            Position::new(5, 7),
            Some(Piece::new(PieceKind::Rook, PieceColor::Black)),
        );
        
        let state = GameState::new(board, PieceColor::White, 0b0000, None, 0, 0);
        
        let moves = get_moves(&state, PieceColor::White);
        let moves = prune_moves_into_check(moves, &state);

        moves.assert_len(5);
        moves.assert_forall(|m| m.to().rank() != 5);
    }

    #[test]
    fn test_prune_moves_into_check_prevent_king_capture_into_check() {
        let mut board = Board::empty();
        board.set(
            Position::new(4, 3),
            Some(Piece::new(PieceKind::King, PieceColor::White)),
        );
        board.set(
            Position::new(5, 7),
            Some(Piece::new(PieceKind::Rook, PieceColor::Black)),
        );
        board.set(
            Position::new(5, 0),
            Some(Piece::new(PieceKind::Rook, PieceColor::Black)),
        );
        board.set(
            Position::new(5, 3),
            Some(Piece::new(PieceKind::Pawn, PieceColor::Black)),
        );
        
        let state = GameState::new(board, PieceColor::White, 0b0000, None, 0, 0);
        
        let moves = get_moves(&state, PieceColor::White);
        let moves = prune_moves_into_check(moves, &state);        
        moves.assert_len(3);
        moves.assert_forall(|m| m.to().rank() != 5);
    }

    #[test]
    fn test_prune_moves_into_check_prevent_castle_through_check() {
        let mut board = Board::empty();
        board.set(
            Position::new(0, 4),
            Some(Piece::new(PieceKind::King, PieceColor::White)),
        );
        board.set(
            Position::new(0, 0),
            Some(Piece::new(PieceKind::Rook, PieceColor::White)),
        );
        board.set(
            Position::new(0, 7),
            Some(Piece::new(PieceKind::Rook, PieceColor::White)),
        );
        board.set(
            Position::new(1, 4),
            Some(Piece::new(PieceKind::Pawn, PieceColor::White)),
        );
        board.set(
            Position::new(7, 3),
            Some(Piece::new(PieceKind::Rook, PieceColor::Black)),
        );
        board.set(
            Position::new(7, 5),
            Some(Piece::new(PieceKind::Rook, PieceColor::Black)),
        );

        let state = GameState::new(board, PieceColor::White, 0b1100, None, 0, 0);
        
        let moves = get_moves(&state, PieceColor::White);
        let moves = prune_moves_into_check(moves, &state);
        
        moves.assert_forall(|m| !matches!(m, Move::Castle{..}));
    }

    #[test]
    fn test_prune_moves_into_check_prevent_castle_out_of_check() {
        let mut board = Board::empty();
        board.set(
            Position::new(0, 4),
            Some(Piece::new(PieceKind::King, PieceColor::White)),
        );
        board.set(
            Position::new(0, 0),
            Some(Piece::new(PieceKind::Rook, PieceColor::White)),
        );
        board.set(
            Position::new(0, 7),
            Some(Piece::new(PieceKind::Rook, PieceColor::White)),
        );
        board.set(
            Position::new(7, 4),
            Some(Piece::new(PieceKind::Rook, PieceColor::Black)),
        );

        let state = GameState::new(board, PieceColor::White, 0b1100, None, 0, 0);        
        let moves = get_moves(&state, PieceColor::White);
        let moves = prune_moves_into_check(moves, &state);
        
        moves.assert_forall(|m| !matches!(m, Move::Castle{..}));
    }

    #[test]
    fn test_prune_moves_into_check_prevent_castle_into_check() {
        let mut board = Board::empty();
    }

}
