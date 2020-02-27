use ansi_escapes::*;
use bus::Bus;

use std::fmt::Display;
use std::io::Write;
use std::net::*;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;

const TICK: Duration = Duration::from_millis(100);

const DIMS: (i16, i16) = (23, 80);

type Coord = (u16, u16);

fn display<T: Write, D>(out: &mut T, esc: D, ball: char, flush: bool)
                        -> Result<(), Box<dyn std::error::Error>>
    where D: Display
{
    write!(out, "{}{}", esc, ball)?;
    if flush {
            out.flush()?;
    }
    Ok(())
}

fn client<W: Write>(mut r: bus::BusReader<Coord>, mut s: W)
          -> Result<(), Box<dyn std::error::Error>>
{
    let mut x0 = 0;
    let mut y0 = 0;
    loop {
        let (x, y) = r.recv()?;
        display(&mut s, CursorTo::AbsoluteXY(x0, y0), ' ', true)?;
        display(&mut s, CursorTo::AbsoluteXY(x, y), '*', false)?;
        x0 = x;
        y0 = y;
    }
}

fn demo(s: Arc<Mutex<Bus<Coord>>>) {
    let mut x = 1;
    let mut y = 1;
    let mut dx = 1;
    let mut dy = 1;
    loop {
        sleep(TICK);
        let mut s = s.lock().unwrap();
        let _ = s.try_broadcast((x as u16, y as u16));
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
    let bus = Arc::new(Mutex::new(Bus::new(5)));
    let s = bus.clone();
    let _ = std::thread::spawn(move || {
        demo(s);
    });
    loop {
        match listener.accept() {
            Ok((mut socket, addr)) => {
                println!("new client: {:?}", addr);
                write!(socket, "{}{}", ClearScreen, CursorHide).unwrap();
                socket.flush().unwrap();
                let r = bus.lock().unwrap().add_rx();
                let _ = std::thread::spawn(move || {
                    client(r, socket).unwrap_or_else(|e| {
                        eprintln!("{:?}: {}", addr, e);
                    });
                });
            }
            Err(e) => {
                println!("couldn't get client: {:?}", e);
            }
        }
    }
}
