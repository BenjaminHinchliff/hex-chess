use crate::{
    coord::Coord,
    piece::{Name, Piece, Team},
};
use std::{collections::HashMap, error::Error, fmt};

type Hex = (Coord, Piece);

const STARTING_PIECES: &[(Coord, Piece)] = &[
    (Coord::new(0, -5), Piece::new(Name::Bishop, Team::White)),
    (Coord::new(0, -4), Piece::new(Name::Bishop, Team::White)),
    (Coord::new(0, -3), Piece::new(Name::Bishop, Team::White)),
    (Coord::new(1, -5), Piece::new(Name::King, Team::White)),
    (Coord::new(-1, -4), Piece::new(Name::Queen, Team::White)),
    (Coord::new(-2, -3), Piece::new(Name::Knight, Team::White)),
    (Coord::new(2, -5), Piece::new(Name::Knight, Team::White)),
    (Coord::new(-3, -2), Piece::new(Name::Rook, Team::White)),
    (Coord::new(3, -5), Piece::new(Name::Rook, Team::White)),
    (Coord::new(4, -5), Piece::new(Name::Pawn, Team::White)),
    (Coord::new(3, -4), Piece::new(Name::Pawn, Team::White)),
    (Coord::new(2, -3), Piece::new(Name::Pawn, Team::White)),
    (Coord::new(1, -2), Piece::new(Name::Pawn, Team::White)),
    (Coord::new(0, -1), Piece::new(Name::Pawn, Team::White)),
    (Coord::new(-1, -1), Piece::new(Name::Pawn, Team::White)),
    (Coord::new(-2, -1), Piece::new(Name::Pawn, Team::White)),
    (Coord::new(-3, -1), Piece::new(Name::Pawn, Team::White)),
    (Coord::new(-4, -1), Piece::new(Name::Pawn, Team::White)),
];

fn reflect_team<'a>(pieces: impl Iterator<Item = Hex> + 'a) -> impl Iterator<Item = Hex> + 'a {
    pieces.map(|(p, piece)| (p.reflect_q(), piece.clone().flip_team()))
}

#[derive(Debug, thiserror::Error, Clone, Copy, PartialEq, Eq)]
pub enum MoveErrorType {
    #[error("{0}")]
    NoPiece(#[from] GetError),
    #[error("Invalid Move for {0}")]
    InvalidMove(Piece),
    #[error("{0} collided with on path")]
    CollisionOnPath(Piece),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MoveError {
    pub err_type: MoveErrorType,
    pub from: Coord,
    pub to: Coord,
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

impl Error for MoveError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

#[derive(Debug, thiserror::Error, Clone, Copy, PartialEq, Eq)]
pub enum GetError {
    #[error("No Piece at position {0}")]
    NoPiece(Coord),
}

#[derive(Debug, Clone)]
pub struct HexBoard {
    pieces: HashMap<Coord, Piece>,
    checkers: [Vec<Coord>; 2],
}

impl HexBoard {
    const N: i32 = 5;

    pub fn new() -> HexBoard {
        HexBoard {
            pieces: HashMap::new(),
            checkers: Default::default(),
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

    #[allow(dead_code)]
    pub fn place(&mut self, c: Coord, piece: Piece) {
        self.pieces.insert(c, piece);
    }

    pub fn get(&self, c: Coord) -> Result<&Piece, GetError> {
        self.pieces.get(&c).ok_or_else(|| GetError::NoPiece(c))
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
            let axial_len = ((v.norm_squared() / 3) as f32).sqrt() as i32;
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

    fn update_checkers(&mut self) {
        return;
        let kings = self.pieces.iter().filter(|(_c, p)| p.name == Name::King);
        for (&pos, king) in kings {
            let mut checkers = Vec::new();
            let enemy_coords = self
                .pieces
                .iter()
                .filter(|(c, p)| p.team == king.team.flip());
            for (&enemy_pos, enemy) in enemy_coords {
                if self.unchecked_can_move(enemy, enemy_pos, pos).is_ok() {
                    checkers.push(pos);
                }
            }
            self.checkers[king.team as usize] = checkers;
        }
    }

    const ADJACENTS: &[Coord] = &[
        Coord::new(1, 0),
        Coord::new(1, -1),
        Coord::new(0, -1),
        Coord::new(-1, 0),
        Coord::new(-1, 1),
        Coord::new(0, 1),
    ];

    pub fn can_move(&self, from: Coord, to: Coord) -> Result<(), MoveError> {
        let piece = self.get(from).map_err(|e| MoveError {
            err_type: e.into(),
            from,
            to,
        })?;

        if self.checkers[piece.team as usize].is_empty() {
            self.unchecked_can_move(piece, from, to)
        } else {
            // are we out of check after the move?
            let mut projected = self.clone();
            projected.teleport(from, to);
            projected.update_checkers();
            if projected.checkers[piece.team as usize].is_empty() {
                Ok(())
            } else {
                Err(MoveError {
                    err_type: MoveErrorType::InvalidMove(*piece),
                    from,
                    to,
                })
            }
        }
    }

    fn unchecked_can_move(&self, piece: &Piece, from: Coord, to: Coord) -> Result<(), MoveError> {
        // is the destination in bounds?
        if to.q.abs() > Self::N || to.r.abs() > Self::N || to.s().abs() > Self::N {
            return Err(MoveError {
                err_type: MoveErrorType::InvalidMove(*piece),
                from,
                to,
            });
        }

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

        Ok(())
    }

    pub fn move_piece(&mut self, from: Coord, to: Coord) -> Result<(), MoveError> {
        self.can_move(from, to)?;

        self.teleport(from, to);

        self.update_checkers();
        Ok(())
    }

    fn teleport(&mut self, from: Coord, to: Coord) {
        let piece = self.pieces.remove(&from).unwrap();
        self.pieces.insert(to, piece);
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
        let _board = HexBoard::new();
    }

    // check that a move is valid and that the piece has the state expected
    fn check_move(board: &mut HexBoard, f: Coord, t: Coord, start_piece: Piece, end_piece: Piece) {
        assert_eq!(board.get(f), Ok(&start_piece), "state:\n{}", board);
        assert_eq!(board.move_piece(f, t), Ok(()));
        assert_eq!(board.get(t), Ok(&end_piece), "state:\n{}", board);
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
            assert_eq!(board.get(f), Ok(&piece), "state:\n{}", board);
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
                err_type: GetError::NoPiece((0, 0).into()).into(),
                from: (0, 0).into(),
                to: (1, 0).into(),
            },
        );
    }

    #[test]
    fn move_pawn() {
        let mut board = HexBoard::new();
        let pawn = Piece::new(Name::Pawn, Team::White);
        board.place((0, -2).into(), pawn.clone());
        board.place((-1, -1).into(), pawn);
        board.place((1, 1).into(), Piece::new(Name::Pawn, Team::Black));

        // move one
        check_move(
            &mut board,
            (0, -2).into(),
            (0, -1).into(),
            Piece::new(Name::Pawn, Team::White),
            Piece::new(Name::Pawn, Team::White),
        );

        // move 2
        check_move(
            &mut board,
            (-1, -1).into(),
            (-1, 1).into(),
            Piece::new(Name::Pawn, Team::White),
            Piece::new(Name::Pawn, Team::White),
        );

        // move 3 fails
        check_move_fails(
            &mut board,
            (-1, 1).into(),
            (-1, 3).into(),
            Some(Piece::new(Name::Pawn, Team::White)),
            MoveError {
                err_type: MoveErrorType::InvalidMove(Piece::new(Name::Pawn, Team::White)),
                from: (-1, 1).into(),
                to: (-1, 3).into(),
            },
        );

        // cannot move diagonally
        check_move_fails(
            &mut board,
            (-1, 1).into(),
            (0, 1).into(),
            Some(Piece::new(Name::Pawn, Team::White)),
            MoveError {
                err_type: MoveErrorType::InvalidMove(Piece::new(Name::Pawn, Team::White)),
                from: (-1, 1).into(),
                to: (0, 1).into(),
            },
        );

        // move black (reflected over q axis)
        check_move(
            &mut board,
            (1, 1).into(),
            (1, -1).into(),
            Piece::new(Name::Pawn, Team::Black),
            Piece::new(Name::Pawn, Team::Black),
        );

        // capture
        check_move(
            &mut board,
            (0, -1).into(),
            (1, -1).into(),
            Piece::new(Name::Pawn, Team::White),
            Piece::new(Name::Pawn, Team::White),
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
