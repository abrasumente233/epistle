use std::net::UdpSocket;

fn main() {
    let socket = UdpSocket::bind("127.0.0.1:23123").expect("Bind failed");
    socket.connect("127.0.0.1:4444").expect("Connection failed");

    let mut s = String::new();
    std::io::stdin().read_line(&mut s).unwrap();
    socket.send(s.as_bytes()).unwrap();

    let mut buf = [0; 10];
    socket.recv(&mut buf).unwrap();

    let recv = String::from_utf8_lossy(&buf);
    print!("{}", recv);
}
