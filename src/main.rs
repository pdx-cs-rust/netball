use ansi_escapes::*;

use std::convert::TryInto;
use std::io::{stdout, Write};
use std::thread::sleep;
use std::time::Duration;

const TICK: Duration = Duration::from_millis(100);

const DIMS: (i16, i16) = (23, 80);

fn main() {
    let mut x = 1;
    let mut y = 1;
    let mut dx = 1;
    let mut dy = 1;
    let mut stdout = stdout();
    print!("{}{}", ClearScreen, CursorHide);
    stdout.flush().unwrap();
    loop {
        let cx = x.try_into().unwrap();
        let cy = y.try_into().unwrap();
        print!("{}*", CursorTo::AbsoluteXY(cx, cy));
        stdout.flush().unwrap();
        sleep(TICK);
        print!("{} ", CursorTo::AbsoluteXY(cx, cy));
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
