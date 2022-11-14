use crate::coord::Coord;
use num_derive::ToPrimitive;
use once_cell::sync::OnceCell;
use std::{collections::HashSet, fmt};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MovesPossible {
    pub _move: bool,
    pub capture: bool,
}

static PAWN_DOUBLES: OnceCell<HashSet<Coord>> = OnceCell::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq, ToPrimitive)]
pub enum Name {
    King,
    Queen,
    Bishop,
    Knight,
    Rook,
    Pawn,
}

impl Name {
    fn verify_pawn(&self, f: Coord, t: Coord) -> Option<MovesPossible> {
        let doubles = PAWN_DOUBLES.get_or_init(|| {
            let mut doubles = HashSet::new();
            doubles.extend(&[
                Coord::new(-4, 1),
                Coord::new(-3, 1),
                Coord::new(-2, 1),
                Coord::new(-1, 1),
                Coord::new(0, 1),
                Coord::new(1, 0),
                Coord::new(1, -1),
                Coord::new(1, -2),
                Coord::new(1, -3),
            ]);
            doubles
        });
        // check trying to move one space forward, or two spaces forward
        if f.q == t.q && (f.r + 1 == t.r || (doubles.contains(&t) && f.r + 2 == t.r)) {
            Some(MovesPossible {
                _move: true,
                capture: false,
            })
        } else if (f.q + 1 == t.q && f.r == t.r) || (f.q - 1 == t.q && f.r + 1 == t.r) {
            Some(MovesPossible {
                _move: false,
                capture: true,
            })
        } else {
            None
        }
    }

    fn verify_bishop(&self, f: Coord, t: Coord) -> Option<MovesPossible> {
        const MOVEMENTS: &[Coord] = &[
            Coord::new(1, -2),
            Coord::new(2, -1),
            Coord::new(1, 1),
            Coord::new(-1, 2),
            Coord::new(-2, 1),
            Coord::new(-1, -1),
        ];
        let v = t - f;
        for &m in MOVEMENTS {
            let f = v / m;
            // check that the movement requested is a non-zero integer multiple of the movement
            // vector
            if f.q == f.r && f * m == v {
                return Some(MovesPossible {
                    _move: true,
                    capture: true,
                });
            }
        }
        None
    }

    fn verify_rook(&self, f: Coord, t: Coord) -> Option<MovesPossible> {
        if f.q == t.q || f.r == t.r || f.s() == t.s() {
            Some(MovesPossible {
                _move: true,
                capture: true,
            })
        } else {
            None
        }
    }

    fn verify_knight(&self, f: Coord, t: Coord) -> Option<MovesPossible> {
        let v = t - f;
        if (v.q * v.r * v.s()).abs() == 6 {
            Some(MovesPossible {
                _move: true,
                capture: true,
            })
        } else {
            None
        }
    }

    fn verify_king(&self, f: Coord, t: Coord) -> Option<MovesPossible> {
        let v = t - f;
        if v.length() == 1 || (v.norm_squared() % 3 == 0 && v.norm_squared() / 3 == 1) {
            Some(MovesPossible {
                _move: true,
                capture: true,
            })
        } else {
            None
        }
    }

    pub fn verify_move(&self, f: Coord, t: Coord) -> Option<MovesPossible> {
        match self {
            Name::Pawn => self.verify_pawn(f, t),
            Name::Bishop => self.verify_bishop(f, t),
            Name::Rook => self.verify_rook(f, t),
            Name::Knight => self.verify_knight(f, t),
            Name::Queen => self.verify_rook(f, t).or(self.verify_bishop(f, t)),
            Name::King => self.verify_king(f, t),
        }
    }
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Name::Pawn => "pawn",
                Name::Bishop => "bishop",
                Name::Rook => "rook",
                Name::Knight => "knight",
                Name::Queen => "queen",
                Name::King => "king",
            }
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ToPrimitive)]
pub enum Team {
    White,
    Black,
}

impl Team {
    pub const fn flip(self) -> Team {
        match self {
            Team::White => Team::Black,
            Team::Black => Team::White,
        }
    }
}

impl fmt::Display for Team {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Team::White => "white",
                Team::Black => "black",
            }
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Piece {
    pub name: Name,
    pub team: Team,
}

impl Piece {
    pub const fn new(name: Name, team: Team) -> Piece {
        Piece { name, team }
    }

    pub const fn flip_team(mut self) -> Self {
        self.team = self.team.flip();
        self
    }

    pub fn verify_move(&self, mut f: Coord, mut t: Coord) -> Option<MovesPossible> {
        if let Team::Black = self.team {
            f = f.reflect_q();
            t = t.reflect_q();
        }

        self.name.verify_move(f, t)
    }
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let c = match self.team {
            Team::White => match self.name {
                Name::Pawn { .. } => '♙',
                Name::Knight => '♘',
                Name::Bishop => '♗',
                Name::Rook => '♖',
                Name::Queen => '♕',
                Name::King => '♔',
            },
            Team::Black => match self.name {
                Name::Pawn { .. } => '♟',
                Name::Knight => '♞',
                Name::Bishop => '♝',
                Name::Rook => '♜',
                Name::Queen => '♛',
                Name::King => '♚',
            },
        };

        write!(f, "{}", c)
    }
}
