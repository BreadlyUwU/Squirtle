use std::io::Read;
use std::os::unix::net::{UnixListener, UnixStream};

fn read_message(stream: &mut UnixStream) -> std::io::Result<String> {
    let mut buf = String::new();
    stream.read_to_string(&mut buf)?;
    Ok(buf.trim().to_string())
}

pub fn start() -> std::io::Result<()> {
    let listener = UnixListener::bind("/run/squirtled.sock")?;

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

                if message == "skibidi" {
                    log::debug!("SKIBIDI DOP DOP YES YES")
                } else {
                    log::debug!("Incoming UNIX connexion: {:?}", message);
                }
                // thread::spawn(|| handle_client(stream));
            }
            Err(err) => {
                log::error!("UNIX socket error: {}", err);
                break;
            }
        }
    }
    Ok(())
}
