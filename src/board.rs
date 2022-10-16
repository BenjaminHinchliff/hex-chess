use crate::coords_utils::axial_to_cube;
use crate::coords_utils::reflect_q;
use crate::piece::{Name, Piece, Team};
use glam::IVec3;
use std::{collections::HashMap, fmt};

type Coord = IVec3;
type Hex = (Coord, Piece);

const STARTING_PIECES: &[((i32, i32), Piece)] = &[
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

fn build_team<'a>(
    pieces: impl Iterator<Item = ((i32, i32), Piece)> + 'a,
) -> impl Iterator<Item = Hex> + 'a {
    pieces.map(|((q, r), p)| (IVec3::new(q, r, -q - r), p))
}

fn reflect_team<'a>(pieces: impl Iterator<Item = Hex> + 'a) -> impl Iterator<Item = Hex> + 'a {
    pieces.map(|(p, piece)| (reflect_q(p), piece.clone().flip_team()))
}

#[derive(Debug, Clone)]
pub struct HexBoard {
    pieces: HashMap<Coord, Piece>,
}

fn is_axis(IVec3 { x, y, z }: IVec3) -> bool {
    (x != 0 && y != 0 && z == 0) || (x != 0 && y == 0 && z != 0) || (x == 0 && y != 0 && z != 0)
}

impl HexBoard {
    const N: i32 = 5;

    pub fn new() -> HexBoard {
        let mut pieces = HashMap::new();

        let team: Vec<_> = build_team(STARTING_PIECES.iter().cloned()).collect();
        pieces.extend(team.clone());
        pieces.extend(reflect_team(team.into_iter()));

        HexBoard { pieces }
    }

    pub fn collides(&self, f: Coord, t: Coord) -> bool {
        let v = t - f;
        // movement along an axis
        if is_axis(v) {
            let axial_len = v.abs().max_element();
            let uv = v / axial_len;
            for n in 1..=axial_len {
                let vn = f + uv * n;
                if self.pieces.contains_key(&vn) {
                    return true;
                }
            }
            false
        } else {
            true
        }
    }

    pub fn move_piece(&mut self, f: Coord, t: Coord) {
        let piece = self.pieces.get(&f).unwrap();
        // can the piece do that?
        if !piece.verify_move(f, t) {
            panic!("help invalid move");
        }
        if self.collides(f, t) {
            panic!("collided");
        }
        let mut piece = self.pieces.remove(&f).unwrap();
        piece.mark_moved();
        self.pieces.insert(t, piece);
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

                match self.pieces.get(&(x, y, -x - y).into()) {
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
    use glam::IVec2;

    #[test]
    fn new() {
        let board = HexBoard::new();

        println!("{}", board);
    }

    fn check_move(board: &mut HexBoard, f: IVec2, t: IVec2, start_piece: Piece, end_piece: Piece) {
        let (f, t) = (axial_to_cube(f), axial_to_cube(t));
        assert_eq!(
            board.pieces.get(&f),
            Some(&start_piece),
            "state:\n{}",
            board
        );
        board.move_piece(f, t);
        assert_eq!(board.pieces.get(&t), Some(&end_piece), "state:\n{}", board);
    }

    fn check_move_sym(board: &mut HexBoard, f: IVec2, t: IVec2, piece: Piece) {
        check_move(board, f, t, piece, piece)
    }

    #[test]
    fn move_pawn() {
        let mut board = HexBoard::new();

        check_move(
            &mut board,
            (0, -1).into(),
            (0, 0).into(),
            Piece::new(Name::Pawn { has_moved: false }, Team::Black),
            Piece::new(Name::Pawn { has_moved: true }, Team::Black),
        );

        check_move(
            &mut board,
            (-1, -1).into(),
            (-1, 1).into(),
            Piece::new(Name::Pawn { has_moved: false }, Team::Black),
            Piece::new(Name::Pawn { has_moved: true }, Team::Black),
        );

        panic!("{}", board);
    }
}
