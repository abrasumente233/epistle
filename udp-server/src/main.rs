use std::net::UdpSocket;

fn main() {
    let socket = UdpSocket::bind("127.0.0.1:4444").expect("Bind failed");

    let mut buf = [0; 10];
    let (_amt, src) = socket.recv_from(&mut buf).unwrap();

    let recv = String::from_utf8_lossy(&buf);
    print!("{}", recv);

    let mut s = String::new();
    std::io::stdin().read_line(&mut s).unwrap();
    socket.send_to(s.as_bytes(), &src).unwrap();
}
