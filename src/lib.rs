#[macro_use]
extern crate error_chain;

use std::io::BufRead;

pub mod errors;
pub use errors::*;

#[derive(Debug, PartialEq)]
pub enum Event {
    Start(Block),
    End(Block),
    Key(String),
    String(String),
    Number(N),
    Bool(bool),
    Null,
}

#[derive(Debug, PartialEq)]
pub enum N {
    Float(f64),
    Int(i64),
    Uint(u64),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Block {
    Object,
    Array,
}

#[derive(Debug, Clone, Copy)]
enum ContainerType {
    Object,
    Array,
    Root,
}

#[derive(Debug, Clone, Copy)]
pub enum ExpectType {
    Colon,
    Comma,
    Key,
    Value,
}

#[derive(Debug, Clone, Copy)]
pub struct Container {
    t: ContainerType,
    expect: ExpectType,
}

impl Container {
    fn object() -> Container {
        Container {
            t: ContainerType::Object,
            expect: ExpectType::Key,
        }
    }

    fn array() -> Container {
        Container {
            t: ContainerType::Array,
            expect: ExpectType::Value,
        }
    }

    fn root() -> Container {
        Container {
            t: ContainerType::Root,
            expect: ExpectType::Value,
        }
    }
}

#[derive(Debug)]
struct JsonReader<B: BufRead> {
    r: B,
    tmp: Option<u8>,
    line: usize,
}

impl<B: BufRead> JsonReader<B> {
    fn new(r: B) -> JsonReader<B> {
        JsonReader {
            r: r,
            tmp: None,
            line: 0,
        }
    }

    fn next(&mut self) -> Result<Option<u8>> {
        if let Some(c) = self.tmp {
            self.tmp = None;
            return Ok(Some(c));
        }
        let mut b = [0];
        if self.r.read(&mut b)? == 0 {
            return Ok(None);
        }
        let c = b[0];
        if c == b'\n' {
            self.line += 1;
        }
        Ok(Some(c))
    }

    fn next_must(&mut self) -> Result<u8> {
        self.next()?
            .map(Ok)
            .or(Some(Err("EOF".into())))
            .unwrap()
    }

    fn push_back(&mut self, c: u8) {
        assert!(self.tmp.is_none());
        self.tmp = Some(c);
    }

    fn take_while<F>(&mut self, buf: &mut Vec<u8>, f: F) -> Result<usize>
        where F: Fn(u8) -> bool
    {
        let mut n = 0;
        loop {
            if let Some(c) = self.next()? {
                if f(c) {
                    buf.push(c);
                    n += 1;
                } else {
                    self.push_back(c);
                    return Ok(n);
                }
            } else {
                return Err("EOF".into());
            }
        }
    }
}

#[derive(Debug)]
pub struct Parser<B: BufRead> {
    reader: JsonReader<B>,
    containers: Vec<Container>,
}

impl<B: BufRead> Parser<B> {
    pub fn from_reader(reader: B) -> Parser<B> {
        Parser {
            reader: JsonReader::new(reader),
            containers: vec![Container::root()],
        }
    }

    #[inline]
    fn container(&mut self) -> &mut Container {
        self.containers.last_mut().unwrap()
    }

    fn start_object(&mut self) -> Result<Event> {
        if let ExpectType::Value = self.container().expect {
            self.containers.push(Container::object());
            Ok(Event::Start(Block::Object))
        } else {
            Err(ErrorKind::Syntax(self.container().expect, '{').into())
        }
    }

    fn end_object(&mut self) -> Result<Event> {
        if let ContainerType::Object = self.container().t {
            let _ = self.containers.pop();
            self.container().expect = ExpectType::Comma;
            Ok(Event::End(Block::Object))
        } else {
            Err(ErrorKind::Syntax(self.container().expect, '}').into())
        }
    }

    fn start_array(&mut self) -> Result<Event> {
        if let ExpectType::Value = self.container().expect {
            self.containers.push(Container::array());
            Ok(Event::Start(Block::Array))
        } else {
            Err(ErrorKind::Syntax(self.container().expect, '[').into())
        }
    }

    fn end_array(&mut self) -> Result<Event> {
        if let ContainerType::Array = self.container().t {
            let _ = self.containers.pop();
            self.container().expect = ExpectType::Comma;
            Ok(Event::End(Block::Array))
        } else {
            Err(ErrorKind::Syntax(self.container().expect, ']').into())
        }
    }

    fn expect(&mut self, v: &[u8]) -> Result<bool> {
        for c in v {
            match self.reader.next()? {
                Some(x) => {
                    if x != *c {
                        return Ok(false);
                    }
                }
                None => return Err("EOF".into()),
            }
        }
        Ok(true)
    }

    fn read_true(&mut self) -> Result<Event> {
        if let ExpectType::Value = self.container().expect {
            if self.expect(b"true")? {
                Ok(Event::Bool(true))
            } else {
                Err("".into())
            }
        } else {
            Err(ErrorKind::Syntax(self.container().expect, 't').into())
        }
    }

    fn read_false(&mut self) -> Result<Event> {
        if let ExpectType::Value = self.container().expect {
            if self.expect(b"false")? {
                Ok(Event::Bool(false))
            } else {
                Err("".into())
            }
        } else {
            Err(ErrorKind::Syntax(self.container().expect, 't').into())
        }
    }

    fn read_null(&mut self) -> Result<Event> {
        if let ExpectType::Value = self.container().expect {
            if self.expect(b"null")? {
                Ok(Event::Null)
            } else {
                Err("".into())
            }
        } else {
            Err(ErrorKind::Syntax(self.container().expect, 't').into())
        }
    }

    fn parse_hex_escape(&mut self) -> Result<u16> {
        let mut n = 0;
        for _ in 0..4 {
            let c = self.reader.next_must()?;
            n = match c {
                c @ b'0'...b'9' => n * 16_u16 + ((c as u16) - (b'0' as u16)),
                b'a' | b'A' => n * 16_u16 + 10_u16,
                b'b' | b'B' => n * 16_u16 + 11_u16,
                b'c' | b'C' => n * 16_u16 + 12_u16,
                b'd' | b'D' => n * 16_u16 + 13_u16,
                b'e' | b'E' => n * 16_u16 + 14_u16,
                b'f' | b'F' => n * 16_u16 + 15_u16,
                _ => {
                    return Err(format!("invalid escape: {}", c as char).into());
                }
            };
        }
        Ok(n)
    }

    fn parse_string_unicode_tail(&mut self, head: u16, buf: &mut Vec<u8>) -> Result<()> {
        match self.parse_hex_escape()? {
            tail @ 0xDC00...0xDFFF => {
                let n = (((head - 0xD800) as u32) << 10 | (tail - 0xDC00) as u32) + 0x1_0000;

                match std::char::from_u32(n as u32) {
                    Some(c) => buf.append(&mut c.to_string().into_bytes()),
                    None => {
                        return Err("".into());
                    }
                }
            }
            0xD800...0xDBFF => return Err("".into()),
            _ => return Err("".into()),
        }
        Ok(())
    }

    fn parse_string_unicode(&mut self, buf: &mut Vec<u8>) -> Result<()> {
        match self.parse_hex_escape()? {
            0xDC00...0xDFFF => return Err("".into()),
            head @ 0xD800...0xDBFF => {
                if !self.expect(b"\\u")? {
                    return Err("".into());
                }
                self.parse_string_unicode_tail(head, buf)?;
            }
            n => {
                match std::char::from_u32(n as u32) {
                    Some(c) => buf.append(&mut c.to_string().into_bytes()),
                    None => {
                        return Err("".into());
                    }
                }
            }
        }
        Ok(())
    }

    fn parse_string_escape(&mut self, buf: &mut Vec<u8>) -> Result<()> {
        match self.reader.next_must()? {
            c @ b'"' | c @ b'\\' | c @ b'/' => buf.push(c),
            b'b' => buf.push(b'\x08'),
            b'f' => buf.push(b'\x0c'),
            b'n' => buf.push(b'\n'),
            b'r' => buf.push(b'\r'),
            b't' => buf.push(b'\t'),
            b'u' => self.parse_string_unicode(buf)?,
            _ => return Err("Illegal escaped characters".into()),
        }

        Ok(())
    }

    fn parse_string(&mut self, buf: &mut Vec<u8>) -> Result<()> {
        loop {
            let _n = self.reader
                .take_while(buf, |c| match c {
                    b'\\' | b'"' => false,
                    _ => true,
                })?;
            match self.reader
                      .next()?
                      .unwrap() {
                b'\\' => self.parse_string_escape(buf)?,
                b'"' => break,
                _ => unreachable!(),
            }
        }
        Ok(())
    }

    fn read_string(&mut self) -> Result<Event> {
        match self.container().expect {
            ExpectType::Key | ExpectType::Value => {
                let mut buf = Vec::new();
                self.parse_string(&mut buf)?;
                let s = String::from_utf8(buf)?;
                if let ExpectType::Key = self.container().expect {
                    self.container().expect = ExpectType::Colon;
                    Ok(Event::Key(s))
                } else {
                    self.container().expect = ExpectType::Comma;
                    Ok(Event::String(s))
                }
            }
            _ => return Err(ErrorKind::Syntax(self.container().expect, '"').into()),
        }
    }

    fn read_number(&mut self, minus: bool) -> Result<Event> {
        let c = self.reader.next_must()?;
        if c == b'0' {
            match self.reader.next()? {
                Some(b'0'...b'9') => return Err("found superfluous leading zero".into()),
                Some(b'.') => self.reader.push_back(b'.'),
                None | _ => {
                    if !minus {
                        return Ok(Event::Number(N::Uint(0)));
                    } else {
                        return Err("found -0".into());
                    }
                }
            }
        } else {
            self.reader.push_back(c);
        }
        let mut decimal_places: Option<i32> = None;
        let mut e: Option<i32> = None;
        let mut e_minus = false;
        let mut tmp: u64 = 0;
        while let Some(c) = self.reader.next()? {
            match c {
                b'0'...b'9' => {
                    if let Some(ev) = e {
                        e = Some(ev * 10_i32 + (c - b'0') as i32);
                    } else {
                        tmp = tmp * 10_u64 + (c - b'0') as u64;
                        if let Some(n) = decimal_places {
                            decimal_places = Some(n + 1);
                        }
                    }
                }
                b'.' => {
                    if decimal_places.is_some() {
                        return Err("".into());
                    }
                    if e.is_some() {
                        return Err("".into());
                    }
                    let nc = self.reader.next_must()?;
                    if nc < b'0' || nc > b'9' {
                        return Err("".into());
                    }
                    self.reader.push_back(nc);
                    decimal_places = Some(0);
                }
                b'e' | b'E' => {
                    if e.is_some() {
                        return Err("".into());
                    }
                    let nc = self.reader.next_must()?;
                    match nc {
                        b'0'...b'9' => self.reader.push_back(nc),
                        b'+' => (),
                        b'-' => e_minus = true,
                        _ => return Err("".into()),
                    }
                    e = Some(0);
                }
                _ => {
                    self.reader.push_back(c);
                    break;
                }
            }
        }

        let index = e.map(|e| if e_minus { -e } else { e }).unwrap_or(0) -
                    decimal_places.unwrap_or(0);
        let n = if index >= 0 {
            let n = tmp * power_of_ten(index as usize);
            if minus {
                N::Int(-(n as i64))
            } else {
                N::Uint(n)
            }
        } else {
            let n = tmp as f64 / power_of_ten(-index as usize) as f64;
            if minus { N::Float(-n) } else { N::Float(n) }
        };

        Ok(Event::Number(n))
    }
}

impl<B: BufRead> Iterator for Parser<B> {
    type Item = Result<Event>;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(c) = match self.reader.next() {
                  Ok(x) => x,
                  Err(e) => return Some(Err(e)),
              } {
            let r = match c {
                b'{' => self.start_object(),
                b'}' => self.end_object(),
                b'[' => self.start_array(),
                b']' => self.end_array(),
                b'"' => self.read_string(),
                b':' => {
                    if let ExpectType::Colon = self.container().expect {
                        self.container().expect = ExpectType::Value;
                        continue;
                    } else {
                        Err(ErrorKind::Syntax(self.container().expect, c as char).into())
                    }
                }
                b',' => {
                    if let ExpectType::Comma = self.container().expect {
                        match self.container().t {
                            ContainerType::Object => self.container().expect = ExpectType::Key,
                            ContainerType::Array => self.container().expect = ExpectType::Value,
                            ContainerType::Root => panic!(),
                        }
                        continue;
                    } else {
                        Err(ErrorKind::Syntax(self.container().expect, c as char).into())
                    }
                }
                b' ' | b'\r' | b'\n' | b'\t' => {
                    // skip whitespace
                    continue;
                }
                c => {
                    if let ExpectType::Value = self.container().expect {
                        self.reader.push_back(c);
                        let r = match c {
                            b't' => self.read_true(),
                            b'f' => self.read_false(),
                            b'n' => self.read_null(),
                            b'-' => {
                                let _ = self.reader.next();
                                self.read_number(true)
                            }
                            b'0'...b'9' => self.read_number(false),
                            c => panic!(c),
                        };
                        self.container().expect = ExpectType::Comma;
                        r
                    } else {
                        Err(ErrorKind::Syntax(self.container().expect, c as char).into())
                    }
                }
            };
            return Some(r);
        }
        None
    }
}

fn power_of_ten(p: usize) -> u64 {
    if p == 0 {
        return 1;
    }
    let mut r = 10;
    for _ in 1..p {
        r *= 10;
    }
    r
}
