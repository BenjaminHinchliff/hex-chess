use std::io::{self, BufRead, Write};

use hex_chess::HexBoard;

pub fn main() {
    let mut board = HexBoard::new_initialize();

    let mut lines = io::stdin().lock().lines();

    loop {
        println!("{}", board);

        print!("Move: ");
        io::stdout().flush().expect("unable to flush output");

        let _move = match lines.next() {
            Some(_move) => _move,
            None => break,
        };

        let _move = _move.expect("unable to read input");
        let _move = _move
            .splitn(2, "->")
            .map(|c| {
                c.trim()
                    .splitn(2, ",")
                    .map(|n| n.trim().parse::<i32>().unwrap())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let (f, t) = if let [f, t] = &_move[..] {
            (f, t)
        } else {
            unreachable!()
        };

        board
            .move_piece((f[0], f[1]).into(), (t[0], t[1]).into())
            .unwrap();
    }
}
