use ansi_escapes::*;

use std::convert::TryInto;
use std::io::Write;
use std::net::*;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;

const TICK: Duration = Duration::from_millis(100);

const DIMS: (i16, i16) = (23, 80);

fn demo(clients: Arc<Mutex<Vec<TcpStream>>>) {
    let mut x = 1;
    let mut y = 1;
    let mut dx = 1;
    let mut dy = 1;
    loop {
        let cx = x.try_into().unwrap();
        let cy = y.try_into().unwrap();
        for out in clients.lock().unwrap().iter_mut() {
            write!(out, "{}*", CursorTo::AbsoluteXY(cx, cy)).unwrap();
            out.flush().unwrap();
        }
        sleep(TICK);
        for out in clients.lock().unwrap().iter_mut() {
            write!(out, "{} ", CursorTo::AbsoluteXY(cx, cy)).unwrap();
        }
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
    let clients = Arc::new(Mutex::new(Vec::new()));
    let demo_clients = clients.clone();
    let _ = std::thread::spawn(move || {
        demo(demo_clients);
    });
    loop {
        match listener.accept() {
            Ok((mut socket, addr)) => {
                println!("new client: {:?}", addr);
                write!(socket, "{}{}", ClearScreen, CursorHide).unwrap();
                socket.flush().unwrap();
                clients.lock().unwrap().push(socket);
            }
            Err(e) => {
                println!("couldn't get client: {:?}", e);
            }
        }
    }
}
