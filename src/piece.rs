use crate::coord::Coord;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MovesPossible {
    pub _move: bool,
    pub capture: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Name {
    Pawn { has_moved: bool },
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
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

    pub fn verify_move(&self, f: Coord, t: Coord) -> Option<MovesPossible> {
        match self {
            Name::Pawn { has_moved } => self.verify_pawn(*has_moved, f, t),
            Name::Bishop => self.verify_bishop(f, t),
            Name::Rook => self.verify_rook(f, t),
            Name::Queen => self.verify_rook(f, t).or(self.verify_bishop(f, t)),
            _ => unimplemented!(),
        }
    }

    // pub const fn moves(&self) -> &[(i32, i32)] {
    //     match self {
    //         Name::Pawn { has_moved } => &[(1, 0)],
    //         _ => unimplemented!(),
    //     }
    // }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Piece {
    name: Name,
    team: Team,
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
            *has_moved = true;
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
