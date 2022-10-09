use std::{collections::HashMap, fmt};

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

type Hex = ((i32, i32), Piece);

const STARTING_PIECES: &[Hex] = &[
    ((0, -5), Piece::new(Name::Bishop, Team::Black)),
    ((0, -4), Piece::new(Name::Bishop, Team::Black)),
    ((0, -3), Piece::new(Name::Bishop, Team::Black)),
    ((1, -5), Piece::new(Name::King, Team::Black)),
    ((-1, -4), Piece::new(Name::Queen, Team::Black)),
    ((-2, -3), Piece::new(Name::Knight, Team::Black)),
    ((2, -5), Piece::new(Name::Knight, Team::Black)),
    ((-3, -2), Piece::new(Name::Rook, Team::Black)),
    ((3, -5), Piece::new(Name::Rook, Team::Black)),
    ((4, -5), Piece::new(Name::pawn(), Team::Black)),
    ((3, -4), Piece::new(Name::pawn(), Team::Black)),
    ((2, -3), Piece::new(Name::pawn(), Team::Black)),
    ((1, -2), Piece::new(Name::pawn(), Team::Black)),
    ((0, -1), Piece::new(Name::pawn(), Team::Black)),
    ((-1, -1), Piece::new(Name::pawn(), Team::Black)),
    ((-2, -1), Piece::new(Name::pawn(), Team::Black)),
    ((-3, -1), Piece::new(Name::pawn(), Team::Black)),
    ((-4, -1), Piece::new(Name::pawn(), Team::Black)),
];

fn reflect_team(pieces: &[Hex]) -> impl Iterator<Item = Hex> + '_ {
    pieces
        .iter()
        .map(|((q, r), p)| ((*q, -q - r), p.clone().flip_team()))
}

#[derive(Debug, Clone)]
pub struct HexBoard {
    pieces: HashMap<(i32, i32), Piece>,
}

impl HexBoard {
    const N: i32 = 5;

    pub fn new() -> HexBoard {
        let mut pieces = HashMap::new();
        pieces.extend(STARTING_PIECES.iter().cloned());
        pieces.extend(reflect_team(STARTING_PIECES));

        HexBoard { pieces }
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
                match self
                    .pieces
                    .get(&(col + 0.max(Self::N - row) - Self::N, row - Self::N))
                {
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

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn board_new() {
//         let board = HexBoard::new();

//         println!("{}", board);
//         panic!();
//     }
// }
