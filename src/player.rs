use crate::board;
use Players::*;

pub trait Player {
    fn play(&self, board: &board::Board) -> (u8, u8);
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Players {
    Player1,
    Player2,
}

impl Players {
    pub fn other(&self) -> Players {
        match self {
            Player1 => Player2,
            Player2 => Player1,
        }
    }
}

impl std::fmt::Display for Players {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let symbol = match self {
            Player1 => 'X',
            Player2 => 'O',
        };
        write!(f, "{}", symbol)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indexing() {
        assert_eq!(Players::Player1 as usize, 0);
        assert_eq!(Players::Player2 as usize, 1);

        assert_eq!(Players::Player1 as u64, 0);
        assert_eq!(Players::Player2 as u64, 1);
    }
}
