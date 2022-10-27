use thiserror::Error;

use crate::{
    coord::Coord,
    piece::{Name, Piece, Team},
};
use std::{collections::HashMap, fmt};

type Hex = (Coord, Piece);

const STARTING_PIECES: &[(Coord, Piece)] = &[
    (Coord::new(0, -5), Piece::new(Name::Bishop, Team::Black)),
    (Coord::new(0, -4), Piece::new(Name::Bishop, Team::Black)),
    (Coord::new(0, -3), Piece::new(Name::Bishop, Team::Black)),
    (Coord::new(1, -5), Piece::new(Name::King, Team::Black)),
    (Coord::new(-1, -4), Piece::new(Name::Queen, Team::Black)),
    (Coord::new(-2, -3), Piece::new(Name::Knight, Team::Black)),
    (Coord::new(2, -5), Piece::new(Name::Knight, Team::Black)),
    (Coord::new(-3, -2), Piece::new(Name::Rook, Team::Black)),
    (Coord::new(3, -5), Piece::new(Name::Rook, Team::Black)),
    (Coord::new(4, -5), Piece::new(Name::pawn(), Team::Black)),
    (Coord::new(3, -4), Piece::new(Name::pawn(), Team::Black)),
    (Coord::new(2, -3), Piece::new(Name::pawn(), Team::Black)),
    (Coord::new(1, -2), Piece::new(Name::pawn(), Team::Black)),
    (Coord::new(0, -1), Piece::new(Name::pawn(), Team::Black)),
    (Coord::new(-1, -1), Piece::new(Name::pawn(), Team::Black)),
    (Coord::new(-2, -1), Piece::new(Name::pawn(), Team::Black)),
    (Coord::new(-3, -1), Piece::new(Name::pawn(), Team::Black)),
    (Coord::new(-4, -1), Piece::new(Name::pawn(), Team::Black)),
];

fn reflect_team<'a>(pieces: impl Iterator<Item = Hex> + 'a) -> impl Iterator<Item = Hex> + 'a {
    pieces.map(|(p, piece)| (p.reflect_q(), piece.clone().flip_team()))
}

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum MoveErrorType {
    #[error("No Piece at starting position")]
    NoPiece,
    #[error("Invalid Move for {0}")]
    InvalidMove(Piece),
    #[error("{0} collided with on path")]
    CollisionOnPath(Piece),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MoveError {
    err_type: MoveErrorType,
    from: Coord,
    to: Coord,
}

impl fmt::Display for MoveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} moving from {} to {}",
            self.err_type, self.from, self.to
        )
    }
}

#[derive(Debug, Clone)]
pub struct HexBoard {
    pieces: HashMap<Coord, Piece>,
}

impl HexBoard {
    const N: i32 = 5;

    pub fn new() -> HexBoard {
        HexBoard {
            pieces: HashMap::new(),
        }
    }

    /// create a new board initialized with both teams from glinski's chess
    pub fn new_initialize() -> HexBoard {
        let mut b = Self::new();

        b.pieces.extend(STARTING_PIECES.iter().cloned());
        b.pieces
            .extend(reflect_team(STARTING_PIECES.iter().cloned()));

        b
    }

    pub fn place(&mut self, c: Coord, piece: Piece) {
        self.pieces.insert(c, piece);
    }

    pub fn get(&self, c: Coord) -> Option<&Piece> {
        self.pieces.get(&c)
    }

    fn collides(&self, f: Coord, t: Coord) -> bool {
        let v = t - f;
        // movement along an axis
        let (axial_len, uv) = if v.is_axis() {
            let axial_len = v.length();
            let uv = v / axial_len;
            (axial_len, uv)
        } else {
            // bishops only collide with every other hex - maybe need to find
            // a better way to structure this
            let axial_len = v.norm_squared() / 3;
            let uv = v / axial_len;
            (axial_len, uv)
        };

        // never inclusive
        for n in 1..axial_len {
            let vn = f + uv * n;
            if self.pieces.contains_key(&vn) {
                return true;
            }
        }
        false
    }

    pub fn move_piece(&mut self, from: Coord, to: Coord) -> Result<(), MoveError> {
        let piece = self.pieces.get(&from).ok_or_else(|| MoveError {
            err_type: MoveErrorType::NoPiece,
            from,
            to,
        })?;

        // can the piece do that? can it capture or just move or both?
        let possible = piece.verify_move(from, to).ok_or_else(|| MoveError {
            err_type: MoveErrorType::InvalidMove(*piece),
            from,
            to,
        })?;

        // if it can't capture and there is a piece there if can't work
        // if it can't move normally and there isn't a piece there then it can't work
        if (!possible.capture && self.pieces.contains_key(&to))
            || (possible.capture
                && self.pieces.contains_key(&to)
                && self.pieces.get(&to).unwrap().team == piece.team)
            || (!possible._move && !self.pieces.contains_key(&to))
        {
            return Err(MoveError {
                err_type: MoveErrorType::InvalidMove(*piece),
                from,
                to,
            });
        }

        // are there any pieces in the way?
        if self.collides(from, to) {
            return Err(MoveError {
                err_type: MoveErrorType::CollisionOnPath(*piece),
                from,
                to,
            });
        }

        let mut piece = self.pieces.remove(&from).unwrap();
        piece.mark_moved();
        self.pieces.insert(to, piece);

        Ok(())
    }
}

impl Default for HexBoard {
    fn default() -> Self {
        Self::new()
    }
}

fn write_border(f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{:1$}", "", (HexBoard::N + 1) as usize,)?;
    for _ in 0..(HexBoard::N + 2) {
        write!(f, "# ")?;
    }
    Ok(())
}

impl fmt::Display for HexBoard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_border(f)?;
        writeln!(f)?;
        for row in 0..(2 * Self::N + 1) {
            write!(f, "{:1$}#", "", Self::N.abs_diff(row) as usize)?;
            for col in 0..(2 * Self::N + 1 - Self::N.abs_diff(row) as i32) {
                // convert cartesian to axial by adding when offset for initial rows
                // then subtract radius to put (0, 0) in the center
                let x = col + 0.max(Self::N - row) - Self::N;
                let y = row - Self::N;

                match self.pieces.get(&(x, y).into()) {
                    Some(p) => write!(f, " {}", p),
                    None => write!(f, " ."),
                }?
            }
            writeln!(f, " #")?;
        }
        write_border(f)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let board = HexBoard::new();

        println!("{}", board);
    }

    // check that a move is valid and that the piece has the state expected
    fn check_move(board: &mut HexBoard, f: Coord, t: Coord, start_piece: Piece, end_piece: Piece) {
        assert_eq!(board.get(f), Some(&start_piece), "state:\n{}", board);
        assert_eq!(board.move_piece(f, t), Ok(()));
        assert_eq!(board.get(t), Some(&end_piece), "state:\n{}", board);
    }

    // check a move is valid for a piece with the same state before and after the move
    fn check_move_sym(board: &mut HexBoard, f: Coord, t: Coord, piece: Piece) {
        check_move(board, f, t, piece, piece)
    }

    fn check_move_fails(
        board: &mut HexBoard,
        f: Coord,
        t: Coord,
        piece: Option<Piece>,
        error: MoveError,
    ) {
        if let Some(piece) = piece {
            assert_eq!(board.get(f), Some(&piece), "state:\n{}", board);
        }
        assert_eq!(board.move_piece(f, t), Err(error));
    }

    #[test]
    fn move_no_piece_is_err() {
        let mut board = HexBoard::new();
        check_move_fails(
            &mut board,
            (0, 0).into(),
            (1, 0).into(),
            None,
            MoveError {
                err_type: MoveErrorType::NoPiece,
                from: (0, 0).into(),
                to: (1, 0).into(),
            },
        );
    }

    #[test]
    fn move_pawn() {
        let mut board = HexBoard::new();
        let pawn = Piece::new(Name::pawn(), Team::Black);
        board.place((0, -2).into(), pawn.clone());
        board.place((-1, -1).into(), pawn);
        board.place((1, 1).into(), Piece::new(Name::pawn(), Team::White));

        // move one
        check_move(
            &mut board,
            (0, -2).into(),
            (0, -1).into(),
            Piece::new(Name::Pawn { has_moved: false }, Team::Black),
            Piece::new(Name::Pawn { has_moved: true }, Team::Black),
        );

        // move 2
        check_move(
            &mut board,
            (-1, -1).into(),
            (-1, 1).into(),
            Piece::new(Name::Pawn { has_moved: false }, Team::Black),
            Piece::new(Name::Pawn { has_moved: true }, Team::Black),
        );

        // move 3 fails
        check_move_fails(
            &mut board,
            (-1, 1).into(),
            (-1, 3).into(),
            Some(Piece::new(Name::Pawn { has_moved: true }, Team::Black)),
            MoveError {
                err_type: MoveErrorType::InvalidMove(Piece::new(
                    Name::Pawn { has_moved: true },
                    Team::Black,
                )),
                from: (-1, 1).into(),
                to: (-1, 3).into(),
            },
        );

        // cannot move diagonally
        check_move_fails(
            &mut board,
            (-1, 1).into(),
            (0, 1).into(),
            Some(Piece::new(Name::Pawn { has_moved: true }, Team::Black)),
            MoveError {
                err_type: MoveErrorType::InvalidMove(Piece::new(
                    Name::Pawn { has_moved: true },
                    Team::Black,
                )),
                from: (-1, 1).into(),
                to: (0, 1).into(),
            },
        );

        // move white (reflected over q axis)
        check_move(
            &mut board,
            (1, 1).into(),
            (1, -1).into(),
            Piece::new(Name::Pawn { has_moved: false }, Team::White),
            Piece::new(Name::Pawn { has_moved: true }, Team::White),
        );

        // capture
        check_move(
            &mut board,
            (0, -1).into(),
            (1, -1).into(),
            Piece::new(Name::Pawn { has_moved: true }, Team::Black),
            Piece::new(Name::Pawn { has_moved: true }, Team::Black),
        )
    }

    #[test]
    fn move_bishop() {
        let mut board = HexBoard::new();
        let bishop = Piece::new(Name::Bishop, Team::Black);
        board.place((0, 0).into(), bishop);

        check_move_sym(&mut board, (0, 0).into(), (1, 1).into(), bishop);
        check_move_sym(&mut board, (1, 1).into(), (3, -3).into(), bishop);
        check_move_sym(&mut board, (3, -3).into(), (1, -2).into(), bishop);

        // invalid bishop moves fails
        check_move_fails(
            &mut board,
            (1, -2).into(),
            (1, 0).into(),
            Some(bishop),
            MoveError {
                err_type: MoveErrorType::InvalidMove(bishop),
                from: (1, -2).into(),
                to: (1, 0).into(),
            },
        );
    }

    #[test]
    fn move_knight() {
        let mut board = HexBoard::new();
        let knight = Piece::new(Name::Knight, Team::Black);
        board.place((0, 0).into(), knight);

        check_move_sym(&mut board.clone(), (0, 0).into(), (3, -1).into(), knight);

        // knight cannot move in lines
        check_move_fails(
            &mut board,
            (0, 0).into(),
            (3, 0).into(),
            Some(knight),
            MoveError {
                err_type: MoveErrorType::InvalidMove(knight),
                from: (0, 0).into(),
                to: (3, 0).into(),
            },
        )
    }

    #[test]
    fn move_king() {
        let mut board = HexBoard::new();
        let king = Piece::new(Name::King, Team::Black);
        board.place((0, 0).into(), king);

        // bishop single
        check_move_sym(&mut board.clone(), (0, 0).into(), (1, 1).into(), king);

        // normal move
        check_move_sym(&mut board.clone(), (0, 0).into(), (1, 0).into(), king);

        // can't move by more than 1
        check_move_fails(
            &mut board.clone(),
            (0, 0).into(),
            (2, 0).into(),
            Some(king),
            MoveError {
                err_type: MoveErrorType::InvalidMove(king),
                from: (0, 0).into(),
                to: (2, 0).into(),
            },
        );
    }
}
