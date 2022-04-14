use epistle::*;
use std::fs::File;
use std::io::Write;
use std::net::TcpStream;
use std::path::Path;

const DOWNLOAD_PREFIX: &str = "Downloads";

fn process_epistle(msg: Epistle) {
    match msg {
        Epistle::Handshake => println!("Handshake!"),
        Epistle::Message(message) => println!("Message: {}", message),
        Epistle::Document(Document {
            filename,
            filesize: _,
            data,
        }) => {
            let saved_path = Path::new(DOWNLOAD_PREFIX).join(filename);
            println!("Received file: {:?}", &saved_path);
            let mut file = File::create(saved_path).unwrap();

            file.write_all(&data).unwrap();
        }
    }
}

fn main() {

    std::fs::create_dir(DOWNLOAD_PREFIX).ok();

    let stream = TcpStream::connect("127.0.0.1:4444").expect("Connection failed");

    let msg: epistle::Epistle =
        rmp_serde::decode::from_read(&stream).expect("Invalid epistle packet");

    process_epistle(msg);
}
