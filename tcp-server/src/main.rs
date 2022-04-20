use std::net::{TcpListener, TcpStream};

fn handle_client(mut stream: TcpStream, filename: &str) -> Result<(), std::io::Error> {
    let bytes = std::fs::read(filename).unwrap();

    let msg = epistle::Epistle::Document(epistle::Document {
        filename: filename.to_string(),
        filesize: bytes.len(),
        data: bytes,
    });

    rmp_serde::encode::write(&mut stream, &msg).unwrap();

    println!("[+] served file {}", filename);

    Ok(())
}

fn main() {
    // Extract filename from command line arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        panic!("please provide a single argument as the filename")
    }
    let filename = &args[1];

    // Fire up server
    let listener = TcpListener::bind("127.0.0.1:4444").expect("Ha");

    for stream in listener.incoming() {
        handle_client(stream.expect("Abnormal connection"), filename)
            .expect("error handling client");
    }
}
