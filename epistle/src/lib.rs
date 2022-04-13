#[macro_use]
extern crate serde_derive;

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Document<'a, 'b> {
    pub filename: &'a str,
    pub filesize: usize,
    pub data: &'b [u8],
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub enum Epistle<'a> {
    Handshake,
    Message(&'a str),
    Document(Document<'a, 'a>),
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
    let val = Epistle::Message("Little tornado");
    let buf = rmp_serde::to_vec(&val).unwrap();
    let deval = rmp_serde::from_read_ref(&buf).unwrap();

    dbg!(&deval);

    assert_eq!(val, deval);
}
