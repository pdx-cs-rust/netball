use ansi_escapes::*;

use std::convert::TryInto;
use std::fmt::Display;
use std::io::Write;
use std::net::*;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;

const TICK: Duration = Duration::from_millis(100);

const DIMS: (i16, i16) = (23, 80);

type Clients = Vec<TcpStream>;

fn display<D>(clients: &mut Clients, esc: D, ball: char, flush: bool)
    where D: Display
{
    let mut fails = Vec::new();
    for (i, out) in clients.iter_mut().enumerate() {
        match write!(out, "{}{}", esc, ball) {
            Ok(_) => (),
            Err(e) => {
                eprintln!("{}", e);
                fails.push(i);
                continue;
            }
        }
        if flush {
            out.flush().unwrap_or_else(|e| {
                eprintln!("{}", e);
                fails.push(i);
            });
        }
    }
    if !fails.is_empty() {
        let mut n = clients.len();
        for i in fails.into_iter().rev() {
            n -= 1;
            clients.swap(i, n);
            clients.pop().unwrap();
        }
    }
}

fn demo(clients: Arc<Mutex<Clients>>) {
    let mut x = 1;
    let mut y = 1;
    let mut dx = 1;
    let mut dy = 1;
    let mut cx = x.try_into().unwrap();
    let mut cy = y.try_into().unwrap();
    loop {
        sleep(TICK);
        let mut clients = clients.lock().unwrap();
        display(&mut clients, CursorTo::AbsoluteXY(cx, cy), ' ', false);
        x += dx;
        y += dy;
        if x <= 0 || x >= DIMS.0 {
            dx = -dx;
        }
        if y <= 0 || y >= DIMS.1 {
            dy = -dy;
        }
        cx = x.try_into().unwrap();
        cy = y.try_into().unwrap();
        display(&mut clients, CursorTo::AbsoluteXY(cx, cy), '*', true);
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
