use async_net_experiment::shitty_block_on;
use async_net::TcpStream;
use futures::io::{AsyncReadExt, AsyncWriteExt};
use futures_timer::Delay;
use futures::StreamExt;
use futures::future::{select, Either};
use futures::pin_mut;
use std::io::{self, BufRead};
use std::thread;
use std::time::Duration;
use windows_sys::Win32::System::Console::SetConsoleTitleW;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

fn set_console_title(title: &str) {
    let wide_title: Vec<u16> = OsStr::new(title).encode_wide().chain(std::iter::once(0)).collect();
    unsafe {
        SetConsoleTitleW(wide_title.as_ptr());
    }
}

fn main() -> io::Result<()> {
    set_console_title("client");

    shitty_block_on(async {
        let mut reconnect_delay = Duration::from_secs(1);
        const MAX_DELAY: Duration = Duration::from_secs(10); // ~12 days // removed 12 days to 10 seconds for practicality LMFAO
        loop {
            match TcpStream::connect("127.0.0.1:8080").await {
                Ok(stream) => {
                    println!("YAY WERE FUCKING ONLINE!");
                    // no reset bs, keep increasing delay on disconnects
                    if let Err(_) = handle_connection(stream).await {
                        println!("NEVERMIND THAT!");
                        Delay::new(reconnect_delay).await;
                        reconnect_delay = (reconnect_delay * 2).min(MAX_DELAY);
                    }
                }
                Err(_) => {
                    println!("NO FUCKING CONNECTION! Retrying in {:?}...", reconnect_delay);
                    Delay::new(reconnect_delay).await;
                    reconnect_delay = (reconnect_delay * 2).min(MAX_DELAY);
                }
            }
        }
    })
}

async fn handle_connection(mut stream: TcpStream) -> io::Result<()> {
    let (tx, mut rx) = futures::channel::mpsc::unbounded();

    // spawn a thread for reading stdin, nothing like stdlib bullshit
    // 
    // but atleast im the real life version of anyone from cyberpunk

    thread::spawn(move || {
        let stdin = std::io::stdin();
        let mut reader = std::io::BufReader::new(stdin);
        let mut line = String::new();
        loop {
            line.clear();
            match reader.read_line(&mut line) {
                Ok(0) => break, // EOF
                Ok(_) => {
                    let msg = line.trim();
                    if !msg.is_empty() {
                        let _ = tx.unbounded_send(msg.to_string());
                    }
                }
                Err(_) => break,
            }
        }
    });

    // dumb loop for messages and pinging the server constantly because why not
    loop {
        let msg_fut = rx.next();
        let delay_fut = Delay::new(Duration::from_secs(5));
        pin_mut!(msg_fut);
        pin_mut!(delay_fut);
        match select(msg_fut, delay_fut).await {
            Either::Left((Some(msg), _)) => {
                let msg_bytes = msg.as_bytes();
                if stream.write_all(msg_bytes).await.is_err() {
                    return Err(io::Error::new(io::ErrorKind::BrokenPipe, "send failed"));
                }
                println!("sent: {:?}", msg_bytes);
                let mut buf = vec![0; 1024];
                match stream.read(&mut buf).await {
                    Ok(0) => return Err(io::Error::new(io::ErrorKind::BrokenPipe, "connection closed")),
                    Ok(n) => println!("received: {:?}", &buf[..n]),
                    Err(_) => return Err(io::Error::new(io::ErrorKind::BrokenPipe, "read failed")),
                }
            }
            Either::Left((None, _)) => return Ok(()),
            Either::Right(_) => {
                if stream.write_all(b"ping").await.is_err() {
                    return Err(io::Error::new(io::ErrorKind::BrokenPipe, "ping send failed"));
                }
                println!("sent ping");
                let mut buf = vec![0; 1024];
                match stream.read(&mut buf).await {
                    Ok(0) => return Err(io::Error::new(io::ErrorKind::BrokenPipe, "connection closed on ping")),
                    Ok(n) => println!("received ping: {:?}", &buf[..n]),
                    Err(_) => return Err(io::Error::new(io::ErrorKind::BrokenPipe, "ping read failed")),
                }
            }
        }
    }
}