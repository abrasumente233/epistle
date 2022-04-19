use std::{
    net::{TcpListener, TcpStream},
    sync::{mpsc::sync_channel, Arc, Mutex},
    thread::spawn,
    time::Duration,
};

use epistle::Epistle;

// What is even this................
fn send_message(stream: &Arc<Mutex<TcpStream>>, msg: String) {
    let stream = &mut *stream.lock().unwrap();
    println!("sending {}", &msg);
    let msg = Epistle::Message(msg);
    rmp_serde::encode::write(stream, &msg).unwrap();
}

fn handle_client(stream: Arc<Mutex<TcpStream>>) -> Result<(), std::io::Error> {
    send_message(&stream, "hello".into());
    std::thread::sleep(Duration::from_secs(3));
    send_message(&stream, "oh no".into());
    std::thread::sleep(Duration::from_secs(3));
    send_message(&stream, "hhh".into());

    Ok(())
}

fn main() {
    // Fire up server
    let listener = TcpListener::bind("127.0.0.1:4444").expect("Cannot bind to :4444");

    let (tx, rx) = sync_channel::<Arc<Mutex<TcpStream>>>(3);

    spawn(move || loop {
        let mut conns = vec![];
        loop {
            if let Ok(res) = rx.try_recv() {
                conns.push(res);
            }

            for conn in conns.iter() {
                let conn = &*conn.lock().unwrap();

                println!("wait on read");
                if let Ok(packet) = rmp_serde::decode::from_read::<_, Epistle>(conn) {
                    dbg!(packet);
                }
                //println!("after")
            }

            //std::thread::sleep(Duration::from_millis(100));
        }
    });

    for stream in listener.incoming() {
        let stream = stream.expect("Abnomral connection");
        //stream
            //.set_read_timeout(Some(Duration::from_millis(50)))
            //.unwrap();
        let stream = Arc::new(Mutex::new(stream));

        /*
        let my_stream = stream.clone();
        spawn(move || {
            handle_client(my_stream).expect("error handling client");
        });
        */

        let my_stream = stream.clone();
        tx.send(my_stream).unwrap();
    }
}
