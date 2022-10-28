use std::io::{self, BufRead, Write};

use hex_chess::Game;

pub fn main() {
    let mut game = Game::new();

    let mut lines = io::stdin().lock().lines();

    loop {
        println!("{}", game);

        print!("Move: ");
        io::stdout().flush().expect("unable to flush output");

        let _move = match lines.next() {
            Some(_move) => _move,
            None => break,
        };

        let _move = _move.expect("unable to read input");
        let _move: Result<Vec<_>, _> = _move
            .splitn(2, "->")
            .map(|c| {
                c.trim()
                    .splitn(2, ",")
                    .map(|n| n.trim().parse::<i32>())
                    .collect::<Result<Vec<_>, _>>()
            })
            .collect();
        let _move = match _move {
            Ok(_move) => _move,
            Err(e) => {
                eprintln!("failed to parse move: {}", e);
                continue;
            }
        };

        let (f, t) = if let [f, t] = &_move[..] {
            (f, t)
        } else {
            // can't get to this point as an invalid move due to parsing logic
            unreachable!()
        };

        if let Err(e) = game.move_piece((f[0], f[1]).into(), (t[0], t[1]).into()) {
            eprintln!("{}", e);
            continue;
        }
    }
}
