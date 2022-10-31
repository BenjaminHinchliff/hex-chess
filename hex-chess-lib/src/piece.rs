use num_derive::ToPrimitive;

use crate::coord::Coord;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MovesPossible {
    pub _move: bool,
    pub capture: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Name {
    King,
    Queen,
    Bishop,
    Knight,
    Rook,
    Pawn { has_moved: bool },
}

impl Name {
    pub const fn pawn() -> Name {
        Name::Pawn { has_moved: false }
    }

    fn verify_pawn(&self, has_moved: bool, f: Coord, t: Coord) -> Option<MovesPossible> {
        // check trying to move one space forward, or two spaces forward
        if f.q == t.q && (f.r + 1 == t.r || (!has_moved && f.r + 2 == t.r)) {
            Some(MovesPossible {
                _move: true,
                capture: false,
            })
        } else if (f.q + 1 == t.q && f.r == t.r) || (f.q == t.q && f.r + 1 == t.r) {
            Some(MovesPossible {
                _move: false,
                capture: true,
            })
        } else {
            None
        }
    }

    fn verify_bishop(&self, f: Coord, t: Coord) -> Option<MovesPossible> {
        if (t - f).norm_squared() % 3 == 0 {
            Some(MovesPossible {
                _move: true,
                capture: true,
            })
        } else {
            None
        }
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
            Name::Pawn { has_moved } => self.verify_pawn(*has_moved, f, t),
            Name::Bishop => self.verify_bishop(f, t),
            Name::Rook => self.verify_rook(f, t),
            Name::Knight => self.verify_knight(f, t),
            Name::Queen => self.verify_rook(f, t).or(self.verify_bishop(f, t)),
            Name::King => self.verify_king(f, t),
        }
    }

    pub const fn idx(&self) -> u8 {
        match self {
            Name::King => 0,
            Name::Queen => 1,
            Name::Bishop => 2,
            Name::Knight => 3,
            Name::Rook => 4,
            Name::Pawn { .. } => 5,
        }
    }

    // pub const fn moves(&self) -> &[(i32, i32)] {
    //     match self {
    //         Name::Pawn { has_moved } => &[(1, 0)],
    //         _ => unimplemented!(),
    //     }
    // }
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Name::Pawn { .. } => "pawn",
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
        if let Team::White = self.team {
            f = f.reflect_q();
            t = t.reflect_q();
        }

        self.name.verify_move(f, t)
    }

    pub fn mark_moved(&mut self) {
        if let Name::Pawn { has_moved } = &mut self.name {
            *has_moved = true
        }
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
