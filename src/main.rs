use ansi_escapes::*;
use bus::Bus;
use netdoor::NetDoor;

use std::fmt::Display;
use std::io::Write;
use std::net::*;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;

const TICK: Duration = Duration::from_millis(100);

type Coord = (f64, f64);

fn display<D>(out: &mut NetDoor, esc: D, ball: char, flush: bool)
                        -> Result<(), Box<dyn std::error::Error>>
    where D: Display
{
    out.write(format!("{}{}", esc, ball).as_bytes())?;
    if flush {
            out.flush()?;
    }
    Ok(())
}

fn client(mut r: bus::BusReader<Coord>, s: TcpStream)
          -> Result<(), Box<dyn std::error::Error>>
{
    let mut door = NetDoor::connect(s, None);
    let (width, height) = if let Ok(true) = door.negotiate_winsize() {
        (door.width.unwrap(), door.height.unwrap())
    } else {
        (80, 23)
    };

    let mut x0 = 0;
    let mut y0 = 0;
    loop {
        let (x, y) = r.recv()?;
        let x = (x * width as f64 + 0.5).floor() as u16;
        let y = (y * height as f64 + 0.5).floor() as u16;
        display(&mut door, CursorTo::AbsoluteXY(y0, x0), ' ', true)?;
        display(&mut door, CursorTo::AbsoluteXY(y, x), '*', false)?;
        x0 = x;
        y0 = y;
    }
}

fn demo(s: Arc<Mutex<Bus<Coord>>>) {
    let mut x = 0.0;
    let mut y = 0.0;
    let mut dx = 0.07;
    let mut dy = 0.03;
    loop {
        sleep(TICK);
        let mut s = s.lock().unwrap();
        let _ = s.try_broadcast((x, y));
        x += dx;
        y += dy;
        if x <= 0.0 {
            x = 0.0;
        }
        if x >= 1.0 {
            x = 1.0;
        }
        if y <= 0.0 {
            y = 0.0;
        }
        if y >= 1.0 {
            y = 1.0;
        }
        if x <= 0.0 || x >= 1.0 {
            dx = -dx;
        }
        if y <= 0.0 || y >= 1.0 {
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
