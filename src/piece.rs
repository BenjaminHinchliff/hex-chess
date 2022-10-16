use std::fmt;

use glam::{IVec2, IVec3};

use crate::coords_utils::reflect_q;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MoveType {
    Move,
    Capture,
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

    pub fn verify_move(&self, f: IVec3, t: IVec3) -> bool {
        match self {
            Name::Pawn { has_moved } => {
                f.x == t.x && (f.y + 1 == t.y || (!has_moved && f.y + 2 == t.y))
            }
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

    pub fn verify_move(&self, mut f: IVec3, mut t: IVec3) -> bool {
        if let Team::White = self.team {
            f = reflect_q(f);
            t = reflect_q(t);
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
