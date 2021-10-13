use std::fmt;
use Players::*;

#[derive(Debug)]
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

impl fmt::Display for Players {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let symbol = match self {
            Player1 => 'X',
            Player2 => 'O',
        };
        write!(f, "{}", symbol)
    }
}
