use std::ops::{Add, Sub};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Position {
    data: u8,
}

impl Position {
    pub fn new(rank: u8, file: u8) -> Self {
        if rank > 7 {
            panic!("Invalid rank: {}", rank)
        }
        if file > 7 {
            panic!("Invalid file: {}", file)
        }

        Self {
            data: (rank << 3) | file,
        }
    }

    #[inline]
    pub fn rank(&self) -> u8 {
        self.data >> 3
    }
    #[inline]
    pub fn file(&self) -> u8 {
        self.data & 0b00000111
    }
}
impl From<Position> for (u8, u8) {
    fn from(position: Position) -> Self {
        (position.data >> 3, position.data & 0b00000111)
    }
}

impl TryFrom<&str> for Position {
    type Error = &'static str;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.len() != 2 {
            return Err("Position must be 2 characters long");
        }

        let mut chars = value.chars();
        let file = match chars.next().unwrap() {
            'a' => 0,
            'b' => 1,
            'c' => 2,
            'd' => 3,
            'e' => 4,
            'f' => 5,
            'g' => 6,
            'h' => 7,
            _ => return Err("Invalid file"),
        };
        let rank = match chars.next().unwrap() {
            '1' => 0,
            '2' => 1,
            '3' => 2,
            '4' => 3,
            '5' => 4,
            '6' => 5,
            '7' => 6,
            '8' => 7,
            _ => return Err("Invalid rank"),
        };

        Ok(Self::new(rank, file))
    }
}

impl From<Position> for usize {
    fn from(position: Position) -> Self {
        position.data as usize
    }
}
impl From<Position> for u8 {
    fn from(position: Position) -> Self {
        position.data
    }
}
impl From<usize> for Position {
    fn from(value: usize) -> Self {
        Self { data: value as u8 }
    }
}
impl From<u8> for Position {
    fn from(value: u8) -> Self {
        Self { data: value }
    }
}
impl ToString for Position {
    fn to_string(&self) -> String {        
        let file = ((self.data & 0b00000111) + ('a' as u8)) as char;
        let rank = ((self.rank() >> 3) + ('1' as u8)) as char;
        format!("{}{}", file, rank)
    }
}
impl std::fmt::Debug for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_string())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_new_ok() {
        let position = Position::new(0, 0);
        assert_eq!(position.rank(), 0);
        assert_eq!(position.file(), 0);

        let position = Position::new(7, 7);
        assert_eq!(position.rank(), 7);
        assert_eq!(position.file(), 7);
    }
    #[test]
    #[should_panic]
    fn test_position_new_panic_invalid_rank() {
        Position::new(8, 0);
    }
    #[test]
    #[should_panic]
    fn test_position_new_panic_invalid_file() {
        Position::new(0, 8);
    }
    #[test]
    fn test_position_from_str_ok() {
        let position = Position::try_from("a1").unwrap();
        assert_eq!(position.rank(), 0);
        assert_eq!(position.file(), 0);

        let position = Position::try_from("b2").unwrap();
        assert_eq!(position.rank(), 1);
        assert_eq!(position.file(), 1);

        let position = Position::try_from("c3").unwrap();
        assert_eq!(position.rank(), 2);
        assert_eq!(position.file(), 2);

        let position = Position::try_from("d4").unwrap();
        assert_eq!(position.rank(), 3);
        assert_eq!(position.file(), 3);

        let position = Position::try_from("e5").unwrap();
        assert_eq!(position.rank(), 4);
        assert_eq!(position.file(), 4);

        let position = Position::try_from("f6").unwrap();
        assert_eq!(position.rank(), 5);
        assert_eq!(position.file(), 5);

        let position = Position::try_from("g7").unwrap();
        assert_eq!(position.rank(), 6);
        assert_eq!(position.file(), 6);

        let position = Position::try_from("h8").unwrap();
        assert_eq!(position.rank(), 7);
        assert_eq!(position.file(), 7);
    }

    #[test]

    fn test_position_from_str_panic_invalid_length() {
        assert!(Position::try_from("a").is_err());
    }
    #[test]
    fn test_position_from_str_panic_invalid_file() {
        assert!(Position::try_from("i1").is_err());
    }
    #[test]
    fn test_position_from_str_panic_invalid_rank() {
        assert!(Position::try_from("a9").is_err());
    }
    #[test]
    fn test_position_to_string_ok() {
        let position = Position::new(0, 0);
        assert_eq!(position.to_string(), "a1");

        let position = Position::new(7, 7);
        assert_eq!(position.to_string(), "h8");
    }
}
