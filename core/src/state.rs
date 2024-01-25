use crate::board::Board;
use crate::moves::{get_moves, prune_moves_into_check, Move};
use crate::piece::{Piece, PieceColor, PieceKind};
use crate::position::Position;

pub enum CastleSide {
    KingSide,
    QueenSide,
}

#[derive(PartialEq, Debug, Clone)]
pub struct GameState {
    pub board: Board,
    pub turn: PieceColor,
    pub castling_rights: u8,
    pub en_passant: Option<Position>,
    pub halfmove_clock: usize,
    pub fullmove_number: usize,
    pub captured_pieces: Vec<Piece>,
}

#[derive(Debug, PartialEq)]
enum FENToken {
    Piece(char),
    Empty(u8),
    EndOfRank,
    EndOfBoard,
    Turn(char),
    Castle(char),
    EnPassant(char, char),
    HalfmoveClock(usize),
    FullmoveNumber(usize),
}

#[derive(Debug)]
pub enum FENParserError {
    NotEnoughArguments,
    InvalidArgument(String),
    InvalidPiece(char),
    InvalidTurn(char),
    InvalidCastle(char),
    InvalidPosition(char, char),
    InvalidRankCount(u8),
    InvalidFileCount(u8),
}

fn fen_char_to_piece(c: char) -> Option<Piece> {
    match c {
        'P' => Some(piece!(Pawn, White)),
        'N' => Some(piece!(Knight, White)),
        'B' => Some(piece!(Bishop, White)),
        'R' => Some(piece!(Rook, White)),
        'Q' => Some(piece!(Queen, White)),
        'K' => Some(piece!(King, White)),

        'p' => Some(piece!(Pawn, Black)),
        'n' => Some(piece!(Knight, Black)),
        'b' => Some(piece!(Bishop, Black)),
        'r' => Some(piece!(Rook, Black)),
        'q' => Some(piece!(Queen, Black)),
        'k' => Some(piece!(King, Black)),
        _ => None,
    }
}

fn lex_fen_str(fen: &str) -> Result<Vec<FENToken>, FENParserError> {
    let mut result = Vec::new();
    let mut args = fen.split_whitespace();
    let mut board_chars = args
        .next()
        .ok_or(FENParserError::NotEnoughArguments)?
        .chars();

    while let Some(c) = board_chars.next() {
        match c {
            '1'..='8' => {
                let n = c.to_digit(10).unwrap() as u8;
                result.push(FENToken::Empty(n));
            }
            '/' => {
                result.push(FENToken::EndOfRank);
            }
            _ => {
                result.push(FENToken::Piece(c));
            }
        }
    }
    result.push(FENToken::EndOfBoard);

    let turn = args.next().ok_or(FENParserError::NotEnoughArguments)?;
    if turn.len() != 1 {
        return Err(FENParserError::InvalidArgument(turn.to_string()));
    }
    result.push(FENToken::Turn(turn.chars().next().unwrap()));

    let castling = args.next().ok_or(FENParserError::NotEnoughArguments)?;

    if castling != "-" {
        let mut castling: Vec<FENToken> = castling.chars().map(|c| FENToken::Castle(c)).collect();
        result.append(&mut castling);
    }

    let mut en_passant = args.next().ok_or(FENParserError::NotEnoughArguments)?;

    if en_passant != "-" {
        if en_passant.len() != 2 {
            return Err(FENParserError::InvalidArgument(en_passant.to_string()));
        }
        let mut en_passant_chars = en_passant.chars();
        let file = en_passant_chars.next().unwrap();
        let rank = en_passant_chars.next().unwrap();

        result.push(FENToken::EnPassant(file, rank));
    }

    let halfmove_clock = args.next().ok_or(FENParserError::NotEnoughArguments)?;

    let halfmove_clock: usize = halfmove_clock
        .parse()
        .map_err(|_| FENParserError::InvalidArgument(halfmove_clock.to_string()))?;

    result.push(FENToken::HalfmoveClock(halfmove_clock));

    let fullmove_number = args.next().ok_or(FENParserError::NotEnoughArguments)?;

    let fullmove_number: usize = fullmove_number
        .parse()
        .map_err(|_| FENParserError::InvalidArgument(fullmove_number.to_string()))?;

    result.push(FENToken::FullmoveNumber(fullmove_number));

    Ok(result)
}

impl GameState {
    pub fn empty() -> Self {
        Self {
            board: Board::empty(),
            turn: PieceColor::White,
            castling_rights: 0b0000,
            en_passant: None,
            halfmove_clock: 0,
            fullmove_number: 1,
            captured_pieces: Vec::new(),
        }
    }
    pub fn new(
        board: Board,
        turn: PieceColor,
        castling_rights: u8,
        en_passant: Option<Position>,
        halfmove_clock: usize,
        fullmove_number: usize,
    ) -> Self {
        Self {
            board,
            turn,
            castling_rights,
            en_passant,
            halfmove_clock,
            fullmove_number,
            captured_pieces: Vec::new(),
        }
    }
    fn parse_fen_tokens(&mut self, tokens: Vec<FENToken>) -> Result<(), FENParserError> {
        let mut rank = 7;
        let mut file = 0;

        let board_squares = self.board.squares_mut();

        for token in tokens {
            match token {
                FENToken::Piece(c) => {
                    if file > 7 {
                        return Err(FENParserError::InvalidFileCount(file));
                    }
                    let piece = fen_char_to_piece(c).ok_or(FENParserError::InvalidPiece(c))?;
                    board_squares[(rank << 3 | file) as usize] = Some(piece);
                    file += 1;
                }
                FENToken::Empty(n) => {
                    file += n;
                    if file > 8 {
                        return Err(FENParserError::InvalidFileCount(file));
                    }
                }
                FENToken::EndOfRank => {
                    if rank == 0 {
                        return Err(FENParserError::InvalidRankCount(9));
                    }
                    rank -= 1;
                    file = 0;
                }
                FENToken::EndOfBoard => {
                    if rank > 0 {
                        return Err(FENParserError::InvalidRankCount(8 - rank));
                    }
                }
                FENToken::Turn(c) => {
                    self.turn = match c {
                        'w' => PieceColor::White,
                        'b' => PieceColor::Black,
                        _ => return Err(FENParserError::InvalidTurn(c)),
                    };
                }
                FENToken::Castle(c) => match c {
                    'K' => self.castling_rights |= 0b1000,
                    'Q' => self.castling_rights |= 0b0100,
                    'k' => self.castling_rights |= 0b0010,
                    'q' => self.castling_rights |= 0b0001,
                    _ => return Err(FENParserError::InvalidCastle(c)),
                },
                FENToken::EnPassant(f, r) => {
                    let pos = Position::try_from(format!("{}{}", f, r).as_str())
                        .map_err(|_| FENParserError::InvalidPosition(f, r))?;
                    self.en_passant = Some(pos);
                }
                FENToken::HalfmoveClock(n) => {
                    self.halfmove_clock = n;
                }
                FENToken::FullmoveNumber(n) => {
                    self.fullmove_number = n;
                }
            }
        }
        Ok(())
    }
    pub fn from_fen(fen: &str) -> Result<Self, FENParserError> {
        let mut result = Self::empty();
        result.parse_fen_tokens(lex_fen_str(fen)?)?;
        Ok(result)
    }
    pub fn default() -> Self {
        Self::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap()
    }
    pub fn can_castle(&self, color: PieceColor, side: CastleSide) -> bool {
        let mask = match (color, side) {
            (PieceColor::White, CastleSide::KingSide) => 0b1000,
            (PieceColor::White, CastleSide::QueenSide) => 0b0100,
            (PieceColor::Black, CastleSide::KingSide) => 0b0010,
            (PieceColor::Black, CastleSide::QueenSide) => 0b0001,
        };
        self.castling_rights & mask != 0
    }
    pub fn unset_castle(&mut self, color: PieceColor, side: CastleSide) {
        let mask = match (color, side) {
            (PieceColor::White, CastleSide::KingSide) => 0b1000,
            (PieceColor::White, CastleSide::QueenSide) => 0b0100,
            (PieceColor::Black, CastleSide::KingSide) => 0b0010,
            (PieceColor::Black, CastleSide::QueenSide) => 0b0001,
        };
        self.castling_rights &= !mask;
    }
    #[cfg(not(tarpaulin_include))]
    pub fn legal_moves(&self) -> Vec<Move> {
        let moves = get_moves(self, self.turn);
        prune_moves_into_check(moves, self)
    }
    pub fn is_in_check(&self) -> bool {
        let king_pos = self.board.find_king(self.turn).unwrap();
        let enemy_color = self.turn.opposite();
        let enemy_moves = get_moves(self, enemy_color);
        enemy_moves.iter().any(|m| m.to() == king_pos)
    }
    #[cfg(not(tarpaulin_include))]
    pub fn is_checkmate(&self) -> bool {
        self.is_in_check() && self.legal_moves().is_empty()
    }
    #[cfg(not(tarpaulin_include))]
    pub fn is_stalemate(&self) -> bool {
        !self.is_in_check() && self.legal_moves().is_empty()
    }
    #[cfg(not(tarpaulin_include))]
    pub fn is_50_move_rule(&self) -> bool {
        self.halfmove_clock >= 100
    }

    fn check_castle_rights_waived(&mut self, pos: Position, piece: Piece) {
        match piece.kind() {
            PieceKind::Rook => {
                if pos.file() == 0 {
                    self.unset_castle(piece.color(), CastleSide::QueenSide);
                } else if pos.file() == 7 {
                    self.unset_castle(piece.color(), CastleSide::KingSide);
                }
            }
            PieceKind::King => {
                self.unset_castle(piece.color(), CastleSide::KingSide);
                self.unset_castle(piece.color(), CastleSide::QueenSide);
            }
            _ => (),
        }
    }
    pub fn apply_move(&mut self, m: Move) {
        self.halfmove_clock += 1;
        self.en_passant = None;

        match m {
            Move::Normal { from, to } => {
                let piece = self.board.get(from).unwrap();
                self.board.set(from, None);
                self.board.set(to, Some(piece));

                if piece.kind() == PieceKind::Pawn {
                    self.halfmove_clock = 0; // pawn moved -> reset halfmove clock
                }

                self.check_castle_rights_waived(from, piece);
            }
            Move::Capture { from, to, captured } => {
                let piece = self.board.get(from).unwrap();
                self.board.set(from, None);
                self.board.set(to, Some(piece));

                self.captured_pieces.push(captured);
                self.halfmove_clock = 0; // capture -> reset halfmove clock

                self.check_castle_rights_waived(from, piece);
                self.check_castle_rights_waived(to, captured);
            }
            Move::Promotion { from, to, promoted } => {
                let piece = self.board.get(from).unwrap();
                self.board.set(from, None);
                self.board.set(to, Some(promoted));

                self.halfmove_clock = 0; // pawn moved -> reset halfmove clock
            }
            Move::PromotionCapture {
                from,
                to,
                captured,
                promoted,
            } => {
                let piece = self.board.get(from).unwrap();
                self.board.set(from, None);
                self.board.set(to, Some(promoted));

                self.captured_pieces.push(captured);
                self.halfmove_clock = 0; // capture -> reset halfmove clock

                self.check_castle_rights_waived(to, captured);
            }
            Move::EnPassant { from, to, captured } => {
                let piece = self.board.get(from).unwrap();
                self.board.set(from, None);
                self.board.set(to, Some(piece));

                let captured_pawn = self.board.get(captured).unwrap();
                self.captured_pieces.push(captured_pawn);
                
                self.board.set(captured, None);
                

                self.halfmove_clock = 0; // capture -> reset halfmove clock
            }
            Move::DoublePawnPush {
                from,
                to,
                en_passant,
            } => {
                let piece = self.board.get(from).unwrap();
                self.board.set(from, None);
                self.board.set(to, Some(piece));

                self.halfmove_clock = 0; // pawn moved -> reset halfmove clock

                self.en_passant = Some(en_passant);
            }
            Move::Castle {
                from,
                to,
                rook_from,
                rook_to,
            } => {
                let king = self.board.get(from).unwrap();
                let rook = self.board.get(rook_from).unwrap();

                self.board.set(from, None);
                self.board.set(to, Some(king));

                self.board.set(rook_from, None);
                self.board.set(rook_to, Some(rook));

                self.unset_castle(king.color(), CastleSide::KingSide);
                self.unset_castle(king.color(), CastleSide::QueenSide);
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test]
    fn test_fen_lexer_firstmove() {
        let fen = "rnbqkbnr/pppppppp/8/8/4P/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
        let tokens = lex_fen_str(fen).unwrap();
        assert_eq!(
            tokens,
            vec![
                FENToken::Piece('r'),
                FENToken::Piece('n'),
                FENToken::Piece('b'),
                FENToken::Piece('q'),
                FENToken::Piece('k'),
                FENToken::Piece('b'),
                FENToken::Piece('n'),
                FENToken::Piece('r'),
                FENToken::EndOfRank,
                FENToken::Piece('p'),
                FENToken::Piece('p'),
                FENToken::Piece('p'),
                FENToken::Piece('p'),
                FENToken::Piece('p'),
                FENToken::Piece('p'),
                FENToken::Piece('p'),
                FENToken::Piece('p'),
                FENToken::EndOfRank,
                FENToken::Empty(8),
                FENToken::EndOfRank,
                FENToken::Empty(8),
                FENToken::EndOfRank,
                FENToken::Empty(4),
                FENToken::Piece('P'),
                FENToken::EndOfRank,
                FENToken::Empty(8),
                FENToken::EndOfRank,
                FENToken::Piece('P'),
                FENToken::Piece('P'),
                FENToken::Piece('P'),
                FENToken::Piece('P'),
                FENToken::Empty(1),
                FENToken::Piece('P'),
                FENToken::Piece('P'),
                FENToken::Piece('P'),
                FENToken::EndOfRank,
                FENToken::Piece('R'),
                FENToken::Piece('N'),
                FENToken::Piece('B'),
                FENToken::Piece('Q'),
                FENToken::Piece('K'),
                FENToken::Piece('B'),
                FENToken::Piece('N'),
                FENToken::Piece('R'),
                FENToken::EndOfBoard,
                FENToken::Turn('b'),
                FENToken::Castle('K'),
                FENToken::Castle('Q'),
                FENToken::Castle('k'),
                FENToken::Castle('q'),
                FENToken::EnPassant('e', '3'),
                FENToken::HalfmoveClock(0),
                FENToken::FullmoveNumber(1)
            ]
        );
    }

    #[test]
    fn test_fen_parser_firstmove() {
        let fen = "rnbqkbnr/pppppppp/8/8/4P/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
        let game_state = GameState::from_fen(fen).unwrap();

        assert_eq!(game_state.board.squares()[0], Some(piece!(Rook, White)));
        assert_eq!(game_state.board.squares()[1], Some(piece!(Knight, White)));
        assert_eq!(game_state.board.squares()[2], Some(piece!(Bishop, White)));
        assert_eq!(game_state.board.squares()[3], Some(piece!(Queen, White)));
        assert_eq!(game_state.board.squares()[4], Some(piece!(King, White)));
        assert_eq!(game_state.board.squares()[5], Some(piece!(Bishop, White)));
        assert_eq!(game_state.board.squares()[6], Some(piece!(Knight, White)));
        assert_eq!(game_state.board.squares()[7], Some(piece!(Rook, White)));

        assert_eq!(game_state.board.squares()[8], Some(piece!(Pawn, White)));
        assert_eq!(game_state.board.squares()[9], Some(piece!(Pawn, White)));
        assert_eq!(game_state.board.squares()[10], Some(piece!(Pawn, White)));
        assert_eq!(game_state.board.squares()[11], Some(piece!(Pawn, White)));
        assert_eq!(game_state.board.squares()[12], None);
        assert_eq!(game_state.board.squares()[13], Some(piece!(Pawn, White)));
        assert_eq!(game_state.board.squares()[14], Some(piece!(Pawn, White)));
        assert_eq!(game_state.board.squares()[15], Some(piece!(Pawn, White)));

        assert_eq!(game_state.board.squares()[24], None);
        assert_eq!(game_state.board.squares()[25], None);
        assert_eq!(game_state.board.squares()[26], None);
        assert_eq!(game_state.board.squares()[27], None);
        assert_eq!(game_state.board.squares()[28], Some(piece!(Pawn, White)));
        assert_eq!(game_state.board.squares()[29], None);
        assert_eq!(game_state.board.squares()[30], None);
        assert_eq!(game_state.board.squares()[31], None);

        assert_eq!(game_state.board.squares()[56], Some(piece!(Rook, Black)));
        assert_eq!(game_state.board.squares()[57], Some(piece!(Knight, Black)));
        assert_eq!(game_state.board.squares()[58], Some(piece!(Bishop, Black)));
        assert_eq!(game_state.board.squares()[59], Some(piece!(Queen, Black)));
        assert_eq!(game_state.board.squares()[60], Some(piece!(King, Black)));
        assert_eq!(game_state.board.squares()[61], Some(piece!(Bishop, Black)));
        assert_eq!(game_state.board.squares()[62], Some(piece!(Knight, Black)));
        assert_eq!(game_state.board.squares()[63], Some(piece!(Rook, Black)));

        for j in 0..8 {
            assert_eq!(game_state.board.squares()[16 + j], None);
            assert_eq!(game_state.board.squares()[32 + j], None);
            assert_eq!(game_state.board.squares()[40 + j], None);
            assert_eq!(
                game_state.board.squares()[48 + j],
                Some(piece!(Pawn, Black))
            );
        }

        assert_eq!(game_state.turn, PieceColor::Black);
        assert_eq!(game_state.castling_rights, 0b1111);
        assert_eq!(game_state.en_passant, Some(Position::new(2, 4)));
        assert_eq!(game_state.halfmove_clock, 0);
        assert_eq!(game_state.fullmove_number, 1);
    }

    #[test]
    fn test_fen_lexer_err_empty_string() {
        let fen = "";
        let tokens = lex_fen_str(fen);
        assert!(tokens.is_err());
    }

    #[test]
    fn test_fen_lexer_err_not_enough_args() {
        let fen = "rnbqkbnr/pppppppp/8/8/4P/8/PPPP1PPP/RNBQKBNR b KQkq e3 0";
        let tokens = lex_fen_str(fen);
        assert!(tokens.is_err());
    }

    #[test]
    fn test_fen_lexer_err_turn_too_many_chars() {
        let fen = "rnbqkbnr/pppppppp/8/8/4P/8/PPPP1PPP/RNBQKBNR bb KQkq e3 0 1";
        let tokens = lex_fen_str(fen);
        assert!(tokens.is_err());
    }

    #[test]
    fn test_fen_lexer_err_en_passant_wrong_length() {
        let fen = "rnbqkbnr/pppppppp/8/8/4P/8/PPPP1PPP/RNBQKBNR b KQkq e33 0 1";
        let tokens = lex_fen_str(fen);
        assert!(tokens.is_err());
    }

    #[test]
    fn test_fen_lexer_err_en_passant_invalid_move_numbers() {
        let fen = "rnbqkbnr/pppppppp/8/8/4P/8/PPPP1PPP/RNBQKBNR b KQkq e3 -1 1";
        let tokens = lex_fen_str(fen);
        assert!(tokens.is_err());

        let fen = "rnbqkbnr/pppppppp/8/8/4P/8/PPPP1PPP/RNBQKBNR b KQkq e3 a 1";
        let tokens = lex_fen_str(fen);
        assert!(tokens.is_err());

        let fen = "rnbqkbnr/pppppppp/8/8/4P/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 -1";
        let tokens = lex_fen_str(fen);
        assert!(tokens.is_err());

        let fen = "rnbqkbnr/pppppppp/8/8/4P/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 b";
        let tokens = lex_fen_str(fen);
        assert!(tokens.is_err());
    }

    #[test]
    fn test_fen_parser_err_invalid_piece() {
        let fen = "xnbqkbnr/pppppppp/8/8/4P/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
        let game_state = GameState::from_fen(fen);
        assert!(game_state.is_err());
    }

    #[test]
    fn test_fen_parser_too_many_ranks() {
        let fen = "rnbqkbnr/pppppppp/8/8/4P/8/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
        let game_state = GameState::from_fen(fen);
        assert!(game_state.is_err());
    }

    #[test]
    fn test_fen_parser_too_few_ranks() {
        let fen = "rnbqkbnr/pppppppp/8/4P/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
        let game_state = GameState::from_fen(fen);
        assert!(game_state.is_err());
    }

    #[test]
    fn test_fen_parser_too_many_files() {
        let fen = "rnbqkbnr/pppppppp/8/8/4P/88/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
        let game_state = GameState::from_fen(fen);
        assert!(game_state.is_err());

        let fen = "rnbqkbnr/ppppppppp/8/8/4P/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
        let game_state = GameState::from_fen(fen);
        assert!(game_state.is_err());
    }

    #[test]
    fn test_fen_parser_err_invalid_turn() {
        let fen = "8/8/8/8/8/8/8/8 x - - 0 0";
        let game_state = GameState::from_fen(fen);
        assert!(game_state.is_err());
    }

    #[test]
    fn test_fen_parser_err_invalid_castle() {
        let fen = "8/8/8/8/8/8/8/8 w XQkq - 0 0";
        let game_state = GameState::from_fen(fen);
        assert!(game_state.is_err());

        let fen = "8/8/8/8/8/8/8/8 w -Qkq - 0 0";
        let game_state = GameState::from_fen(fen);
        assert!(game_state.is_err());
    }

    #[test]
    fn test_fen_parser_err_invalid_en_passant() {
        let fen = "8/8/8/8/8/8/8/8 w - x3 0 0";
        let game_state = GameState::from_fen(fen);
        assert!(game_state.is_err());

        let fen = "8/8/8/8/8/8/8/8 w - e9 0 0";
        let game_state = GameState::from_fen(fen);
        assert!(game_state.is_err());

        let fen = "8/8/8/8/8/8/8/8 w - e0 0 0";
        let game_state = GameState::from_fen(fen);
        assert!(game_state.is_err());

        let fen = "8/8/8/8/8/8/8/8 w - 30 0 0";
        let game_state = GameState::from_fen(fen);
        assert!(game_state.is_err());

        let fen = "8/8/8/8/8/8/8/8 w - ea 0 0";

        let game_state = GameState::from_fen(fen);
        assert!(game_state.is_err());
    }

    #[test]
    fn test_fen_parser_castle_order_doesnt_matter() {
        let fen1 = "8/8/8/8/8/8/8/8 w KQkq - 0 0";
        let game_state1 = GameState::from_fen(fen1).unwrap();

        let fen2 = "8/8/8/8/8/8/8/8 w QKqk - 0 0";
        let game_state2 = GameState::from_fen(fen2).unwrap();

        assert_eq!(game_state1, game_state2);
    }

    #[test]
    fn test_fen_parser_trailing_empty_counter_optional() {
        let fen1 = "p7/1p6/2p5/3p4/4p3/5p2/6p1/ w - - 0 0";
        let game_state1 = GameState::from_fen(fen1).unwrap();

        let fen2 = "p7/1p/2p/3p/4p/5p/6p/8 w - - 0 0";
        let game_state2 = GameState::from_fen(fen2).unwrap();

        assert_eq!(game_state1, game_state2);
    }

    #[test]
    fn test_fen_parser_empty_board() {
        let fen = "/////// w - - 0 0";
        let game_state = GameState::from_fen(fen).unwrap();
        assert_eq!(game_state.board, Board::empty());
    }

    #[test]
    fn test_game_state_default_no_error() {
        GameState::default();
    }

    #[test]
    fn test_game_state_can_castle() {
        let game_state = GameState::default();
        assert!(game_state.can_castle(PieceColor::White, CastleSide::KingSide));
        assert!(game_state.can_castle(PieceColor::White, CastleSide::QueenSide));
        assert!(game_state.can_castle(PieceColor::Black, CastleSide::KingSide));
        assert!(game_state.can_castle(PieceColor::Black, CastleSide::QueenSide));
    }

    #[test]
    fn test_game_state_unset_castle() {
        let mut game_state = GameState::default();
        game_state.unset_castle(PieceColor::White, CastleSide::KingSide);
        assert!(!game_state.can_castle(PieceColor::White, CastleSide::KingSide));
        assert!(game_state.can_castle(PieceColor::White, CastleSide::QueenSide));
        assert!(game_state.can_castle(PieceColor::Black, CastleSide::KingSide));
        assert!(game_state.can_castle(PieceColor::Black, CastleSide::QueenSide));

        game_state.unset_castle(PieceColor::White, CastleSide::QueenSide);
        assert!(!game_state.can_castle(PieceColor::White, CastleSide::KingSide));
        assert!(!game_state.can_castle(PieceColor::White, CastleSide::QueenSide));
        assert!(game_state.can_castle(PieceColor::Black, CastleSide::KingSide));
        assert!(game_state.can_castle(PieceColor::Black, CastleSide::QueenSide));

        game_state.unset_castle(PieceColor::Black, CastleSide::KingSide);
        assert!(!game_state.can_castle(PieceColor::White, CastleSide::KingSide));
        assert!(!game_state.can_castle(PieceColor::White, CastleSide::QueenSide));
        assert!(!game_state.can_castle(PieceColor::Black, CastleSide::KingSide));
        assert!(game_state.can_castle(PieceColor::Black, CastleSide::QueenSide));

        game_state.unset_castle(PieceColor::Black, CastleSide::QueenSide);
        assert!(!game_state.can_castle(PieceColor::White, CastleSide::KingSide));
        assert!(!game_state.can_castle(PieceColor::White, CastleSide::QueenSide));
        assert!(!game_state.can_castle(PieceColor::Black, CastleSide::KingSide));
        assert!(!game_state.can_castle(PieceColor::Black, CastleSide::QueenSide));
    }
    
    #[test_case("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", false; "white not in check default")]
    #[test_case("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1", false; "black not in check default")]
    #[test_case("8/4NP2/5Bp1/1Q6/2Kp1R2/1r2p2b/pq1bk3/1r6 w - - 0 1", false; "random 001 white")]
    #[test_case("8/4NP2/5Bp1/1Q6/2Kp1R2/1r2p2b/pq1bk3/1r6 b - - 0 1", false; "random 001 black")]
    #[test_case("8/2K2n2/p7/2bP1bP1/1B3P1p/8/Q2p1Rk1/1r5N w - - 0 1", false; "random 002 white")]
    #[test_case("8/2K2n2/p7/2bP1bP1/1B3P1p/8/Q2p1Rk1/1r5N b - - 0 1", true; "random 002 black")]
    #[test_case("1b1R1Q2/2P1B1kN/5P1n/8/r3p3/K2p4/1P2PP2/8 w - - 0 1", true; "random 003 white")]
    #[test_case("1b1R1Q2/2P1B1kN/5P1n/8/r3p3/K2p4/1P2PP2/8 b - - 0 1", true; "random 003 black")]
    #[test_case("nR2Q3/N7/1p2P3/2K5/2rP4/2P1pq2/pP1p4/7k w - - 0 1", true; "random 004 white")]
    #[test_case("nR2Q3/N7/1p2P3/2K5/2rP4/2P1pq2/pP1p4/7k b - - 0 1", false; "random 004 black")]
    #[test_case("8/k1b2Rn1/5p2/8/K1Bp1n2/PpP2p1P/8/1rB5 w - - 0 1", false; "random 005 white")]
    #[test_case("8/k1b2Rn1/5p2/8/K1Bp1n2/PpP2p1P/8/1rB5 b - - 0 1", false; "random 005 black")]
    fn test_game_state_is_in_check(state_fen: &str, expected: bool) {
        let state = GameState::from_fen(state_fen).unwrap();
        assert_eq!(state.is_in_check(), expected);
    }
}
