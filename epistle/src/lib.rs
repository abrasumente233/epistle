#[macro_use]
extern crate serde_derive;

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Document {
    pub filename: String,
    pub filesize: usize,
    pub data: Vec<u8>, // Avoid putting large data in memory
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub enum Epistle {
    Handshake,
    Message(String),
    Document(Document),
}

#[test]
fn test_handshake() {
    let val = Epistle::Handshake;
    let buf = rmp_serde::to_vec(&val).unwrap();
    let deval = rmp_serde::from_read_ref(&buf).unwrap();

    dbg!(&deval);

    assert_eq!(val, deval);
}

#[test]
fn test_message() {
    let val = Epistle::Message("Little tornado".into());
    let buf = rmp_serde::to_vec(&val).unwrap();
    let deval = rmp_serde::from_read_ref(&buf).unwrap();

    dbg!(&deval);

    assert_eq!(val, deval);
}
