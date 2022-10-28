use crate::{
    board::{GetError, HexBoard, MoveError},
    coord::Coord,
    piece::Team,
};
use std::fmt;

#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum GameError {
    #[error("{0}")]
    PieceError(#[from] GetError),
    #[error("wrong turn - expected {real} but was given {given}")]
    TurnError { given: Team, real: Team },
    #[error("{0}")]
    MoveError(#[from] MoveError),
}

pub struct Game {
    turn: Team,
    board: HexBoard,
}

impl Game {
    pub fn new() -> Self {
        Self {
            turn: Team::White,
            board: HexBoard::new_initialize(),
        }
    }

    pub fn move_piece(&mut self, from: Coord, to: Coord) -> Result<(), GameError> {
        let piece = self.board.get(from)?;
        if piece.team != self.turn {
            return Err(GameError::TurnError {
                given: piece.team,
                real: self.turn,
            });
        }
        self.board.move_piece(from, to)?;
        self.turn = self.turn.flip();
        Ok(())
    }
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}'s turn", self.turn)?;
        write!(f, "{}", self.board)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn moving_works() {
        let mut game = Game::new();
        assert_eq!(game.move_piece((0, 1).into(), (0, 0).into()), Ok(()));
    }

    #[test]
    fn unable_to_move_wrong_team() {
        let mut game = Game::new();
        assert_eq!(
            game.move_piece((0, -1).into(), (0, 0).into()),
            Err(GameError::TurnError {
                given: Team::Black,
                real: Team::White
            })
        );
    }
}
