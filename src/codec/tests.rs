use std::str::FromStr;

use super::*;
use crate::{types::*, util::*};

#[test]
fn decoder_simple() {
    let mut codec = Codec::new();
    let buf = &mut BytesMut::new();
    buf.reserve(1024);

    buf.put("ping\r\npOnG\r\n".as_bytes());
    assert_eq!(
        codec.decode(buf).unwrap().unwrap().unwrap(),
        ServerMessage::Ping
    );
    assert_eq!(
        codec.decode(buf).unwrap().unwrap().unwrap(),
        ServerMessage::Pong
    );
    assert!(codec.decode(buf).unwrap().is_none());

    buf.put("+Ok\r\nbad message here\r\n".as_bytes());
    buf.put(
        format!(
            "-err '{} test.x.*.y.>'\r\n",
            PERMISSIONS_VIOLATION_FOR_SUBSCRIPTION
        )
        .as_bytes(),
    );
    assert_eq!(
        codec.decode(buf).unwrap().unwrap().unwrap(),
        ServerMessage::Ok
    );
    assert!(codec.decode(buf).unwrap().unwrap().is_err());
    let s = Subject::from_str("test.x.*.y.>").unwrap();
    assert_eq!(
        codec.decode(buf).unwrap().unwrap().unwrap(),
        ServerMessage::Err(ProtocolError::PermissionsViolationForSubscription(s))
    );

    // Invalid utf8
    buf.put(vec![0, 159u8, 146, 150, 10].as_slice());
    assert!(codec.decode(buf).unwrap().unwrap().is_err());

    buf.put("pi".as_bytes());
    assert!(codec.decode(buf).unwrap().is_none());
    buf.put("ng\r".as_bytes());
    assert!(codec.decode(buf).unwrap().is_none());
    buf.put("\n".as_bytes());
    assert_eq!(
        codec.decode(buf).unwrap().unwrap().unwrap(),
        ServerMessage::Ping
    );
}

#[test]
fn decoder_info() {
    let mut codec = Codec::new();
    let buf = &mut BytesMut::new();
    buf.reserve(1024);

    buf.put(
        "INFO {\"server_id\":\"Zk0GQ3JBSrg3oyxCRRlE09\",\"version\":\"1.2.0\",\"proto\":1,\"\
         go\":\"go1.10.3\",\"host\":\"0.0.0.0\",\"port\":4222,\"max_payload\":1048576,\"\
         client_id\":2392}\r\n"
            .as_bytes(),
    );
    assert_eq!(
        codec.decode(buf).unwrap().unwrap().unwrap(),
        ServerMessage::Info(Info {
            server_id: String::from("Zk0GQ3JBSrg3oyxCRRlE09"),
            version: String::from("1.2.0"),
            go: String::from("go1.10.3"),
            host: String::from("0.0.0.0"),
            port: 4222,
            max_payload: 1048576,
            proto: 1,
            client_id: Some(2392),
            auth_required: false,
            tls_required: false,
            tls_verify: false,
            connect_urls: Vec::new(),
        })
    );
}

#[test]
fn decoder_msg() {
    let mut codec = Codec::new();
    let buf = &mut BytesMut::new();
    buf.reserve(1024);

    buf.put("ping\r\nmsg test 0 12\r\nhello w".as_bytes());
    assert_eq!(
        codec.decode(buf).unwrap().unwrap().unwrap(),
        ServerMessage::Ping
    );
    assert!(codec.decode(buf).unwrap().is_none());
    buf.put("orld!\r\n".as_bytes());
    buf.put("msg test 0 5\r\nshort\r\n".as_bytes());
    buf.put("msg test 0 0\r\n\r\n".as_bytes());
    assert_eq!(
        codec.decode(buf).unwrap().unwrap().unwrap(),
        ServerMessage::Msg(Msg::new(
            Subject::from_str("test").unwrap(),
            0,
            None,
            b"hello world!".to_vec()
        ))
    );
    assert_eq!(
        codec.decode(buf).unwrap().unwrap().unwrap(),
        ServerMessage::Msg(Msg::new(
            Subject::from_str("test").unwrap(),
            0,
            None,
            b"short".to_vec()
        ))
    );
    assert_eq!(
        codec.decode(buf).unwrap().unwrap().unwrap(),
        ServerMessage::Msg(Msg::new(
            Subject::from_str("test").unwrap(),
            0,
            None,
            b"".to_vec()
        ))
    );

    buf.put("msg test 0 4\r\nhello world\r\nping\r\n".as_bytes());
    assert!(codec.decode(buf).unwrap().unwrap().is_err());
    assert!(codec.decode(buf).unwrap().unwrap().is_err());
    assert_eq!(
        codec.decode(buf).unwrap().unwrap().unwrap(),
        ServerMessage::Ping
    );

    buf.put("msg test 0 reply 13\r\nhello\r\nworld!\r\n".as_bytes());
    assert_eq!(
        codec.decode(buf).unwrap().unwrap().unwrap(),
        ServerMessage::Msg(Msg::new(
            Subject::from_str("test").unwrap(),
            0,
            Some(Subject::from_str("reply").unwrap()),
            b"hello\r\nworld!".to_vec()
        ))
    );
}
