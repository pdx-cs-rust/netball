use ansi_escapes::*;

use std::convert::TryInto;
use std::io::{stdout, Write};
use std::net::*;
use std::thread::sleep;
use std::time::Duration;

const TICK: Duration = Duration::from_millis(100);

const DIMS: (i16, i16) = (23, 80);

fn demo<W: Write>(mut out: W) {
    let mut x = 1;
    let mut y = 1;
    let mut dx = 1;
    let mut dy = 1;
    let mut stdout = stdout();
    write!(out, "{}{}", ClearScreen, CursorHide).unwrap();
    stdout.flush().unwrap();
    loop {
        let cx = x.try_into().unwrap();
        let cy = y.try_into().unwrap();
        write!(out, "{}*", CursorTo::AbsoluteXY(cx, cy)).unwrap();
        stdout.flush().unwrap();
        sleep(TICK);
        write!(out, "{} ", CursorTo::AbsoluteXY(cx, cy)).unwrap();
        x += dx;
        y += dy;
        if x <= 0 || x >= DIMS.0 {
            dx = -dx;
        }
        if y <= 0 || y >= DIMS.1 {
            dy = -dy;
        }
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:13000").unwrap();
    loop {
        match listener.accept() {
            Ok((socket, addr)) => {
                println!("new client: {:?}", addr);
                let _ = std::thread::spawn(move || {
                    demo(socket);
                });
            }
            Err(e) => {
                println!("couldn't get client: {:?}", e);
            }
        }
    }
}
