use ansi_escapes::*;
use netdoor::NetDoor;

use std::{
    io::Write,
    net::TcpStream,
    os::unix::io::FromRawFd,
};

use async_std::{
    future::Future,
    net::TcpListener,
    os::unix::io::IntoRawFd,
    task,
};
use broadcaster::BroadcastChannel;
use futures_util::StreamExt;
use std::time::Duration;

const TICK: Duration = Duration::from_millis(100);

type Coord = (f64, f64);

type Result<T> =
    std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

fn spawn_and_log_error<F>(fut: F) -> task::JoinHandle<()>
where
    F: Future<Output = Result<()>> + Send + 'static,
{
    task::spawn(async move {
        if let Err(e) = fut.await {
            eprintln!("{}", e)
        }
    })
}

async fn client(mut r: BroadcastChannel<Coord>, mut s: TcpStream) -> Result<()> {
    let (width, height) = {
        let mut door = NetDoor::connect(s.try_clone()?, None);
        if let Ok(true) = door.negotiate_winsize() {
            (door.width.unwrap(), door.height.unwrap())
        } else {
            (80, 23)
        }
    };

    s.write(format!("{}{}", ClearScreen, CursorHide).as_bytes())?;
    s.flush()?;

    let mut x0 = 0;
    let mut y0 = 0;
    let mut x = x0;
    let mut y = y0;
    loop {
        s.write(format!("{} ", CursorTo::AbsoluteXY(y0, x0)).as_bytes())?;
        s.write(format!("{}*", CursorTo::AbsoluteXY(y, x)).as_bytes())?;
        s.flush()?;
        x0 = x;
        y0 = y;
        let (nx, ny) = r.next().await.unwrap();
        x = (nx * width as f64 + 0.5).floor() as u16;
        y = (ny * height as f64 + 0.5).floor() as u16;
    }
}

async fn demo(s: BroadcastChannel<Coord>) {
    let mut x = 0.0;
    let mut y = 0.0;
    let mut dx = 0.07;
    let mut dy = 0.03;
    loop {
        task::sleep(TICK).await;
        let _ = s.send(&(x, y)).await;
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

async fn accept_loop() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:13000").await?;
    let bus = BroadcastChannel::new();
    let s = bus.clone();
    let _ = task::spawn(demo(s));
    let mut incoming = listener.incoming();
    while let Some(socket) = incoming.next().await {
        let socket = socket?;
        let addr = socket.peer_addr()?;
        let fd = socket.clone().into_raw_fd();
        eprintln!("fd: {}", fd);
        let socket = unsafe {
            std::net::TcpStream::from_raw_fd(fd)
        };
        eprintln!("new client: {:?}", addr);
        spawn_and_log_error(client(bus.clone(), socket));
    }
    Ok(())
}

fn main() -> Result<()> {
    task::block_on(accept_loop())
}
