use hex_chess::HexBoard;

pub fn main() {
    let board = HexBoard::new_initialize();

    println!("{}", board);
}
