use crate::{piece::{Piece, PieceKind, PieceColor}, position::Position};

pub type Square = Option<Piece>;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Board {
    squares: [Square; 64],
}

impl Board {
    pub fn empty() -> Self {
        Self {
            squares: [None; 64],
        }
    }    

    pub fn squares(&self) -> &[Square; 64] {
        &self.squares
    }

    pub fn squares_mut(&mut self) -> &mut[Square; 64] {
        &mut self.squares
    }

    pub fn rank(&self, rank: usize) -> &[Square; 8] {
        if rank > 7 { 
            panic!("Invalid rank: {}", rank) 
        }

        self.squares[rank * 8..(rank + 1) * 8].try_into().unwrap()
    }

    pub fn file(&self, file: usize) -> [Square; 8] {
        if file > 7 { 
            panic!("Invalid file: {}", file) 
        }

        let mut result = [None; 8];
        for rank in 0..8 {
            result[rank] = self.squares[rank * 8 + file];
        }
        result
    }

    pub fn get(&self, position: Position) -> Square {
        // bit repr == flattened index :)
        self.squares[usize::from(position)] 
    }
    pub fn set(&mut self, position: Position, square: Square) {       
        self.squares[usize::from(position)] = square;
    }
    pub fn find_king(&self, color: PieceColor) -> Option<Position> {
        self.squares.iter().enumerate().find_map(|(index, square)| {
            square.and_then(|piece| {
                if piece.kind() == PieceKind::King && piece.color() == color {
                    Some(Position::from(index))
                } else {
                    None
                }
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_board_empty_board() {
        let board = Board::empty();
        assert_eq!(board.squares(), &[None; 64]);
    }

    #[test]
    fn test_board_rank_refs() {
        let board = Board::empty();
        assert_eq!(board.rank(0), &board.squares[0..8]);
        assert_eq!(board.rank(1), &board.squares[8..16]);
        assert_eq!(board.rank(2), &board.squares[16..24]);
        assert_eq!(board.rank(3), &board.squares[24..32]);
        assert_eq!(board.rank(4), &board.squares[32..40]);
        assert_eq!(board.rank(5), &board.squares[40..48]);
        assert_eq!(board.rank(6), &board.squares[48..56]);
        assert_eq!(board.rank(7), &board.squares[56..64]);
    }

    #[test]
    #[should_panic]
    fn test_board_rank_out_of_bounds() {
        let board = Board::empty();
        board.rank(8);
    }

    #[test]
    fn test_board_file_refs() {
        let board = Board::empty();
        assert_eq!(board.file(0), [None; 8]);
        assert_eq!(board.file(1), [None; 8]);
        assert_eq!(board.file(2), [None; 8]);
        assert_eq!(board.file(3), [None; 8]);
        assert_eq!(board.file(4), [None; 8]);
        assert_eq!(board.file(5), [None; 8]);
        assert_eq!(board.file(6), [None; 8]);
        assert_eq!(board.file(7), [None; 8]);
    }

    #[test]
    #[should_panic]
    fn test_board_file_out_of_bounds() {
        let board = Board::empty();
        board.file(8);
    }

    #[test]
    fn test_board_find_king() {
        let mut board = Board::empty();
        board.set(Position::try_from("e4").unwrap(), Some(Piece::new(PieceKind::King, PieceColor::White)));
        assert_eq!(board.find_king(PieceColor::White), Some(Position::try_from("e4").unwrap()));
        assert_eq!(board.find_king(PieceColor::Black), None);
    }
}