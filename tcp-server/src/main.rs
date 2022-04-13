use std::fs::File;
use std::io::{copy, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};

fn drain_http_stream(stream: &mut TcpStream) {
    let mut request = vec![];

    loop {
        let mut buf = vec![0u8; 1024];
        let read = stream.read(&mut buf).unwrap();

        // EOF, or the client sent 0 bytes.
        if read == 0 {
            break;
        }

        request.extend_from_slice(&buf[..read]);

        if request.len() > 4 && &request[request.len() - 4..] == b"\r\n\r\n" {
            break;
        }
    }
}

fn handle_client(mut stream: TcpStream, filename: &str) -> Result<(), std::io::Error> {
    drain_http_stream(&mut stream);

    let file = File::open(filename).expect("Cannot open file");
    let size = file.metadata().unwrap().len();

    stream.write_all(b"HTTP/1.1 200 OK\r\n")?;
    stream.write_all(format!("Content-Length: {}\r\n", size).as_bytes())?;
    stream.write_all(b"Content-Type: text/plain\r\n")?;
    stream.write_all(
        format!(
            "Content-Disposition: attachment; filename=\"{}\"\r\n",
            filename
        )
        .as_bytes(),
    )?;
    stream.write_all(b"\r\n")?;

    let mut buf_reader = BufReader::new(file);
    copy(&mut buf_reader, &mut stream)?;

    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        panic!("please provide a single argument as the filename")
    }
    let filename = &args[1];

    let listener = TcpListener::bind("127.0.0.1:4444").expect("Ha");

    for stream in listener.incoming() {
        handle_client(stream.expect("Abnormal connection"), filename)
            .expect("error handling client");
    }
}
