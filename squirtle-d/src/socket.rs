use std::io::{Read, Write};
use std::net::Shutdown::Both;
use std::os::unix::net::{UnixListener, UnixStream};

fn read_message(stream: &mut UnixStream) -> std::io::Result<String> {
    let mut buf = String::new();
    stream.read_to_string(&mut buf)?;
    Ok(buf.trim().to_string())
}

pub fn start(sock: &str) -> std::io::Result<()> {
    match UnixListener::bind(sock) {
        Ok(listener) => {
            println!("SquirtleD started");
            for stream in listener.incoming() {
                match stream {
                    Ok(mut stream) => {
                        let message = match read_message(&mut stream) {
                            Ok(msg) => msg,
                            Err(err) => {
                                log::error!("Failed to read from client: {}", err);
                                continue;
                            }
                        };

                        match message.as_str() {
                            // What the heck is going on
                            "check" => {
                                log::debug!("SKIBIDI DOP DOP YES YES");
                                stream.write_all(b"Ok")?;
                                stream.shutdown(std::net::Shutdown::Write)?;
                            },
                            "stop" => {
                                log::debug!("Received a stop signal");
                                stream.shutdown(Both).expect("Failed to stop the stream :S");
                                log::debug!("Deleting the now unused socket file");
                                std::fs::remove_file(sock).expect("Unable to delete the old UNIX socket file");
                                std::process::exit(0);
                            },
                            _ => log::warn!("Unhandled UNIX connexion: {:?}", message),
                        }
                    }
                    Err(err) => {
                        log::error!("UNIX socket error: {}", err);
                    }
                }
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("Failled to create an UNIX socket at {}: {}", &sock, e);
            Err(e)
        }
    }
}
