use std::{
    net::{TcpListener, TcpStream},
    sync::{
        mpsc::{self, Receiver, SyncSender},
        Arc, Mutex,
    },
    thread::spawn,
};

use epistle::Epistle;

// @FIXME: UPDATE THE NOTES.
// Server accepts client sockets, and for every one of them:
//     1) add it to clients' pool. (via `client_add_chan`)
//     2) read on it, waiting for client messages and broadcast
//        messages to everyone in the pool. (via `broadcast_chan`)

fn broadcast_thread(streams: Arc<Mutex<Vec<TcpStream>>>, bcast_rx: Receiver<Epistle>) {
    loop {
        if let Ok(epistle) = bcast_rx.recv() {
            println!("broadcasting {:?} to everyone", &epistle);
            for mut stream in streams.lock().unwrap().iter() {
                rmp_serde::encode::write(&mut stream, &epistle).unwrap();
            }
        }
    }
}

fn recv_thread(stream: TcpStream, bcast_tx: SyncSender<Epistle>) {
    loop {
        match rmp_serde::decode::from_read::<_, Epistle>(&stream) {
            Ok(epistle) => bcast_tx.send(epistle).unwrap(),
            Err(_) => todo!(), // Disconnect when we received bad packets.
        }
    }
}

fn main() {
    // Fire up server
    let listener = TcpListener::bind("127.0.0.1:4444").expect("Cannot bind to :4444");

    let (bcast_tx, bcast_rx) = mpsc::sync_channel::<Epistle>(3);

    let streams: Vec<TcpStream> = vec![];
    let streams = Arc::new(Mutex::new(streams));

    let bcast_streams = streams.clone();
    spawn(move || broadcast_thread(bcast_streams, bcast_rx));

    for stream in listener.incoming() {
        let reader_stream = stream.expect("Abnomral connection");
        let writer_stream = reader_stream.try_clone().expect("TcpStream clone failed");

        let bcast_tx = bcast_tx.clone();
        spawn(move || recv_thread(reader_stream, bcast_tx));

        streams.lock().unwrap().push(writer_stream);
    }
}
