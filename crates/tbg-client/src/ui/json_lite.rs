//! Minimal JSON reader for tests that pin web data files to `tbg-core`
//! (the client has no JSON dependency and the tests need only values).

use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    List(Vec<Json>),
    Obj(BTreeMap<String, Json>),
}

impl Json {
    pub fn get(&self, key: &str) -> &Json {
        match self {
            Json::Obj(map) => map.get(key).unwrap_or(&Json::Null),
            _ => &Json::Null,
        }
    }

    pub fn list(&self) -> &[Json] {
        match self {
            Json::List(items) => items,
            _ => &[],
        }
    }

    pub fn keys(&self) -> Vec<&str> {
        match self {
            Json::Obj(map) => map.keys().map(String::as_str).collect(),
            _ => Vec::new(),
        }
    }

    /// An integer value; `None` for anything else (fractions included).
    pub fn int(&self) -> Option<i64> {
        match self {
            Json::Num(n) if n.fract() == 0.0 => Some(*n as i64),
            _ => None,
        }
    }

    pub fn str(&self) -> Option<&str> {
        match self {
            Json::Str(s) => Some(s),
            _ => None,
        }
    }
}

pub fn parse(text: &str) -> Result<Json, String> {
    let mut p = Parser {
        s: text.as_bytes(),
        i: 0,
    };
    let value = p.value()?;
    p.ws();
    if p.i == p.s.len() {
        Ok(value)
    } else {
        Err(format!("trailing data at byte {}", p.i))
    }
}

struct Parser<'a> {
    s: &'a [u8],
    i: usize,
}

impl Parser<'_> {
    fn ws(&mut self) {
        while self.s.get(self.i).is_some_and(u8::is_ascii_whitespace) {
            self.i += 1;
        }
    }

    fn eat(&mut self, byte: u8) -> Result<(), String> {
        self.ws();
        if self.s.get(self.i) == Some(&byte) {
            self.i += 1;
            Ok(())
        } else {
            Err(format!("expected '{}' at byte {}", byte as char, self.i))
        }
    }

    fn word(&mut self, word: &str, value: Json) -> Result<Json, String> {
        if self.s[self.i..].starts_with(word.as_bytes()) {
            self.i += word.len();
            Ok(value)
        } else {
            Err(format!("bad literal at byte {}", self.i))
        }
    }

    fn value(&mut self) -> Result<Json, String> {
        self.ws();
        match self.s.get(self.i) {
            Some(b'{') => self.object(),
            Some(b'[') => self.array(),
            Some(b'"') => self.string().map(Json::Str),
            Some(b't') => self.word("true", Json::Bool(true)),
            Some(b'f') => self.word("false", Json::Bool(false)),
            Some(b'n') => self.word("null", Json::Null),
            Some(_) => self.number(),
            None => Err("unexpected end".into()),
        }
    }

    fn object(&mut self) -> Result<Json, String> {
        self.eat(b'{')?;
        let mut map = BTreeMap::new();
        self.ws();
        if self.s.get(self.i) == Some(&b'}') {
            self.i += 1;
            return Ok(Json::Obj(map));
        }
        loop {
            self.ws();
            let key = self.string()?;
            self.eat(b':')?;
            let value = self.value()?;
            if map.insert(key.clone(), value).is_some() {
                return Err(format!("duplicate key {key}"));
            }
            self.ws();
            match self.s.get(self.i) {
                Some(b',') => self.i += 1,
                Some(b'}') => {
                    self.i += 1;
                    return Ok(Json::Obj(map));
                }
                _ => return Err(format!("bad object at byte {}", self.i)),
            }
        }
    }

    fn array(&mut self) -> Result<Json, String> {
        self.eat(b'[')?;
        let mut items = Vec::new();
        self.ws();
        if self.s.get(self.i) == Some(&b']') {
            self.i += 1;
            return Ok(Json::List(items));
        }
        loop {
            items.push(self.value()?);
            self.ws();
            match self.s.get(self.i) {
                Some(b',') => self.i += 1,
                Some(b']') => {
                    self.i += 1;
                    return Ok(Json::List(items));
                }
                _ => return Err(format!("bad array at byte {}", self.i)),
            }
        }
    }

    // Data files only escape quotes and backslashes; anything else is
    // rejected rather than misread.
    fn string(&mut self) -> Result<String, String> {
        self.eat(b'"')?;
        let mut out = Vec::new();
        while let Some(&byte) = self.s.get(self.i) {
            self.i += 1;
            match byte {
                b'"' => return String::from_utf8(out).map_err(|e| e.to_string()),
                b'\\' => {
                    let escaped = *self.s.get(self.i).ok_or("dangling escape")?;
                    self.i += 1;
                    match escaped {
                        b'"' | b'\\' | b'/' => out.push(escaped),
                        _ => return Err(format!("unsupported escape at byte {}", self.i)),
                    }
                }
                _ => out.push(byte),
            }
        }
        Err("unterminated string".into())
    }

    fn number(&mut self) -> Result<Json, String> {
        let start = self.i;
        while self
            .s
            .get(self.i)
            .is_some_and(|b| b.is_ascii_digit() || b"+-.eE".contains(b))
        {
            self.i += 1;
        }
        let text = std::str::from_utf8(&self.s[start..self.i]).map_err(|e| e.to_string())?;
        text.parse::<f64>()
            .map(Json::Num)
            .map_err(|_| format!("bad number {text:?} at byte {start}"))
    }
}
