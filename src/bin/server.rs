use async_net_experiment::shitty_block_on;
use async_net::TcpListener;
use futures::io::{AsyncReadExt, AsyncWriteExt};
use std::io;
use std::thread;
use windows_sys::Win32::System::Console::SetConsoleTitleW;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

// our own runtime cause fuck using tokio or async-std im a masochist
fn set_console_title(title: &str) {
    let wide_title: Vec<u16> = OsStr::new(title).encode_wide().chain(std::iter::once(0)).collect();
    unsafe {
        SetConsoleTitleW(wide_title.as_ptr());
    }
}
fn main() -> io::Result<()> {
    set_console_title("server");

    shitty_block_on(async {
        let listener = TcpListener::bind("127.0.0.1:8080").await?;
        println!("SERVER IS LISTENING ON 127.0.0.1:8080");

        loop {
            let (mut socket, _) = listener.accept().await?;

            // spawn a thread for each connection, dumbb stupid concurrent
            thread::spawn(move || {
                let _ = shitty_block_on(async move {
                    let mut buf = [0; 1024];

                    loop {
                        let n = match socket.read(&mut buf).await {
                            Ok(n) => n,
                            Err(e) => {
                                eprintln!("FAILED FOR READING SOCKET; err = {:?}", e);
                                return;
                            }
                        };

                        if n == 0 {
                            return;
                        }

                        if let Err(e) = socket.write_all(&buf[0..n]).await {
                            eprintln!("FAILED WRITING TO SOCKKET; err = {:?}", e);
                            return;
                        }
                    }
                });
            });
        }
    })
}