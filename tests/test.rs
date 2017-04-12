extern crate jsonpull;

use jsonpull::*;

#[test]
fn test_null() {
    assert_eq!(Parser::from_reader("null".as_bytes()).next().unwrap().unwrap(),
               Event::Null);
}

#[test]
fn test_bool() {
    assert_eq!(Parser::from_reader("true".as_bytes()).next().unwrap().unwrap(),
               Event::Bool(true));
    assert_eq!(Parser::from_reader("false".as_bytes()).next().unwrap().unwrap(),
               Event::Bool(false));
}

#[test]
fn test_number() {
    assert_eq!(Parser::from_reader("0".as_bytes()).next().unwrap().unwrap(),
               Event::Number(N::Uint(0)));
    assert_eq!(Parser::from_reader("10".as_bytes()).next().unwrap().unwrap(),
               Event::Number(N::Uint(10)));
    assert_eq!(Parser::from_reader("-10".as_bytes()).next().unwrap().unwrap(),
               Event::Number(N::Int(-10)));
    assert_eq!(Parser::from_reader("1e0".as_bytes()).next().unwrap().unwrap(),
               Event::Number(N::Uint(1)));
    assert_eq!(Parser::from_reader("1e2".as_bytes()).next().unwrap().unwrap(),
               Event::Number(N::Uint(100)));
    assert_eq!(Parser::from_reader("-1e2".as_bytes()).next().unwrap().unwrap(),
               Event::Number(N::Int(-100)));
    assert_eq!(Parser::from_reader("0.0".as_bytes()).next().unwrap().unwrap(),
               Event::Number(N::Float(0.0)));
    assert_eq!(Parser::from_reader("123.456".as_bytes()).next().unwrap().unwrap(),
               Event::Number(N::Float(123.456)));
    assert_eq!(Parser::from_reader("123.456e3".as_bytes()).next().unwrap().unwrap(),
               Event::Number(N::Uint(123456)));
    assert_eq!(Parser::from_reader("-123.456e3".as_bytes()).next().unwrap().unwrap(),
               Event::Number(N::Int(-123456)));
    assert_eq!(Parser::from_reader("-123.456e2".as_bytes()).next().unwrap().unwrap(),
               Event::Number(N::Float(-12345.6)));
    assert_eq!(Parser::from_reader("1e-2".as_bytes()).next().unwrap().unwrap(),
               Event::Number(N::Float(0.01)));
    assert_eq!(Parser::from_reader("100e+2".as_bytes()).next().unwrap().unwrap(),
               Event::Number(N::Uint(10000)));
    assert_eq!(Parser::from_reader("100e-2".as_bytes()).next().unwrap().unwrap(),
               Event::Number(N::Float(1.0)));
    assert_eq!(Parser::from_reader("-100e+2".as_bytes()).next().unwrap().unwrap(),
               Event::Number(N::Int(-10000)));
    assert_eq!(Parser::from_reader("-100e-2".as_bytes()).next().unwrap().unwrap(),
               Event::Number(N::Float(-1.0)));
}

#[test]
fn test_string() {
    let j = r#""line\nnewline\"\\abc\uD83D\uDC96""#.as_bytes();
    assert_eq!(Parser::from_reader(j).next().unwrap().unwrap(),
               Event::String("line\nnewline\"\\abc💖".to_string()));
}

#[test]
fn test_object() {
    let j = r#"{"key": "value",
                "key2": "value2",
                "object1": {"object_key1": "object_value1"},
                "key3": "value3"}"#
        .as_bytes();
    assert_eq!(Parser::from_reader(j).collect::<Result<Vec<_>>>().unwrap(),
               vec![Event::Start(Block::Object),
                    Event::Key("key".to_string()),
                    Event::String("value".to_string()),
                    Event::Key("key2".to_string()),
                    Event::String("value2".to_string()),
                    Event::Key("object1".to_string()),
                    Event::Start(Block::Object),
                    Event::Key("object_key1".to_string()),
                    Event::String("object_value1".to_string()),
                    Event::End(Block::Object),
                    Event::Key("key3".to_string()),
                    Event::String("value3".to_string()),
                    Event::End(Block::Object)])
}

#[test]
fn test_array() {
    let j = r#"["value1", "value2", ["value3", "value4"], "value5"]"#.as_bytes();
    assert_eq!(Parser::from_reader(j).collect::<Result<Vec<_>>>().unwrap(),
               vec![Event::Start(Block::Array),
                    Event::String("value1".to_string()),
                    Event::String("value2".to_string()),
                    Event::Start(Block::Array),
                    Event::String("value3".to_string()),
                    Event::String("value4".to_string()),
                    Event::End(Block::Array),
                    Event::String("value5".to_string()),
                    Event::End(Block::Array)])
}

#[test]
fn test_total() {
    let j = r#"{"a": 1,
                "b": false,
                "c": null,
                "d": -1.1e-1,
                "e": [
                    { "x": 1, "y": 2 },
                    { "x": 2, "y": 3 },
                ]}"#
            .as_bytes();
    let mut p = Parser::from_reader(j);
    assert_eq!(p.next().unwrap().unwrap(), Event::Start(Block::Object));
    assert_eq!(p.next().unwrap().unwrap(), Event::Key("a".into()));
    assert_eq!(p.next().unwrap().unwrap(), Event::Number(N::Uint(1)));
    assert_eq!(p.next().unwrap().unwrap(), Event::Key("b".into()));
    assert_eq!(p.next().unwrap().unwrap(), Event::Bool(false));
    assert_eq!(p.next().unwrap().unwrap(), Event::Key("c".into()));
    assert_eq!(p.next().unwrap().unwrap(), Event::Null);
    assert_eq!(p.next().unwrap().unwrap(), Event::Key("d".into()));
    assert_eq!(p.next().unwrap().unwrap(), Event::Number(N::Float(-0.11)));
    assert_eq!(p.next().unwrap().unwrap(), Event::Key("e".into()));
    assert_eq!(p.next().unwrap().unwrap(), Event::Start(Block::Array));
    assert_eq!(p.next().unwrap().unwrap(), Event::Start(Block::Object));
    assert_eq!(p.next().unwrap().unwrap(), Event::Key("x".into()));
    assert_eq!(p.next().unwrap().unwrap(), Event::Number(N::Uint(1)));
    assert_eq!(p.next().unwrap().unwrap(), Event::Key("y".into()));
    assert_eq!(p.next().unwrap().unwrap(), Event::Number(N::Uint(2)));
    assert_eq!(p.next().unwrap().unwrap(), Event::End(Block::Object));
    assert_eq!(p.next().unwrap().unwrap(), Event::Start(Block::Object));
    assert_eq!(p.next().unwrap().unwrap(), Event::Key("x".into()));
    assert_eq!(p.next().unwrap().unwrap(), Event::Number(N::Uint(2)));
    assert_eq!(p.next().unwrap().unwrap(), Event::Key("y".into()));
    assert_eq!(p.next().unwrap().unwrap(), Event::Number(N::Uint(3)));
    assert_eq!(p.next().unwrap().unwrap(), Event::End(Block::Object));
    assert_eq!(p.next().unwrap().unwrap(), Event::End(Block::Array));
    assert_eq!(p.next().unwrap().unwrap(), Event::End(Block::Object));
}
