use std::collections::HashSet;

enum DecodedValue {
    Null,
    Boolean(bool),
    Number(f64),
    Text(String),
    Sequence(Vec<DecodedValue>),
    Record(Vec<(String, DecodedValue)>),
}

struct Decoder<'a> {
    source: &'a str,
    offset: usize,
}

pub(crate) fn decode<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    source: &str,
) -> Result<v8::Local<'s, v8::Value>, String> {
    let mut decoder = Decoder { source, offset: 0 };
    decoder.skip_whitespace();
    let value = decoder.read_value()?;
    decoder.skip_whitespace();
    if decoder.offset != source.len() {
        return Err("Unexpected trailing data".to_owned());
    }
    materialize(scope, value)
}

impl Decoder<'_> {
    fn read_value(&mut self) -> Result<DecodedValue, String> {
        self.skip_whitespace();
        match self.peek_byte() {
            Some(b'n') => {
                self.read_keyword("null")?;
                Ok(DecodedValue::Null)
            }
            Some(b't') => {
                self.read_keyword("true")?;
                Ok(DecodedValue::Boolean(true))
            }
            Some(b'f') => {
                self.read_keyword("false")?;
                Ok(DecodedValue::Boolean(false))
            }
            Some(b'"') => self.read_text().map(DecodedValue::Text),
            Some(b'[') => self.read_sequence(),
            Some(b'{') => self.read_record(),
            Some(b'-' | b'0'..=b'9') => self.read_number(),
            _ => Err(format!("Unexpected token at byte {}", self.offset)),
        }
    }

    fn read_sequence(&mut self) -> Result<DecodedValue, String> {
        self.offset += 1;
        self.skip_whitespace();
        let mut values = Vec::new();
        if self.peek_byte() == Some(b']') {
            self.offset += 1;
            return Ok(DecodedValue::Sequence(values));
        }
        loop {
            values.push(self.read_value()?);
            self.skip_whitespace();
            match self.peek_byte() {
                Some(b',') => {
                    self.offset += 1;
                    self.skip_whitespace();
                }
                Some(b']') => {
                    self.offset += 1;
                    return Ok(DecodedValue::Sequence(values));
                }
                _ => return Err("Expected ',' or ']'".to_owned()),
            }
        }
    }

    fn read_record(&mut self) -> Result<DecodedValue, String> {
        self.offset += 1;
        self.skip_whitespace();
        let mut members = Vec::new();
        if self.peek_byte() == Some(b'}') {
            self.offset += 1;
            return Ok(DecodedValue::Record(members));
        }
        loop {
            if self.peek_byte() != Some(b'"') {
                return Err("Expected a quoted member name".to_owned());
            }
            let name = self.read_text()?;
            self.skip_whitespace();
            if self.peek_byte() != Some(b':') {
                return Err("Expected ':' after member name".to_owned());
            }
            self.offset += 1;
            members.push((name, self.read_value()?));
            self.skip_whitespace();
            match self.peek_byte() {
                Some(b',') => {
                    self.offset += 1;
                    self.skip_whitespace();
                }
                Some(b'}') => {
                    self.offset += 1;
                    return Ok(DecodedValue::Record(members));
                }
                _ => return Err("Expected ',' or '}'".to_owned()),
            }
        }
    }

    fn read_text(&mut self) -> Result<String, String> {
        self.offset += 1;
        let mut output = String::new();
        loop {
            let Some(byte) = self.peek_byte() else {
                return Err("Unterminated string".to_owned());
            };
            match byte {
                b'"' => {
                    self.offset += 1;
                    return Ok(output);
                }
                b'\\' => {
                    self.offset += 1;
                    let Some(escape) = self.peek_byte() else {
                        return Err("Unterminated escape sequence".to_owned());
                    };
                    self.offset += 1;
                    match escape {
                        b'"' => output.push('"'),
                        b'\\' => output.push('\\'),
                        b'/' => output.push('/'),
                        b'b' => output.push('\u{0008}'),
                        b'f' => output.push('\u{000c}'),
                        b'n' => output.push('\n'),
                        b'r' => output.push('\r'),
                        b't' => output.push('\t'),
                        b'u' => output.push(self.read_unicode_escape()?),
                        _ => return Err("Invalid escape sequence".to_owned()),
                    }
                }
                0..=31 => return Err("Unescaped control character".to_owned()),
                _ => {
                    let character = self.source[self.offset..]
                        .chars()
                        .next()
                        .ok_or_else(|| "Invalid UTF-8 text".to_owned())?;
                    output.push(character);
                    self.offset += character.len_utf8();
                }
            }
        }
    }

    fn read_unicode_escape(&mut self) -> Result<char, String> {
        let first = self.read_hex_quad()?;
        let scalar = if (0xd800..=0xdbff).contains(&first) {
            if self.source.as_bytes().get(self.offset..self.offset + 2) != Some(b"\\u") {
                return Err("High surrogate is not followed by a low surrogate".to_owned());
            }
            self.offset += 2;
            let second = self.read_hex_quad()?;
            if !(0xdc00..=0xdfff).contains(&second) {
                return Err("Invalid low surrogate".to_owned());
            }
            0x10000 + (((first as u32 - 0xd800) << 10) | (second as u32 - 0xdc00))
        } else if (0xdc00..=0xdfff).contains(&first) {
            return Err("Unexpected low surrogate".to_owned());
        } else {
            first as u32
        };
        char::from_u32(scalar).ok_or_else(|| "Invalid Unicode scalar value".to_owned())
    }

    fn read_hex_quad(&mut self) -> Result<u16, String> {
        let Some(text) = self.source.get(self.offset..self.offset + 4) else {
            return Err("Incomplete Unicode escape".to_owned());
        };
        if !text.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err("Invalid Unicode escape".to_owned());
        }
        self.offset += 4;
        u16::from_str_radix(text, 16).map_err(|_| "Invalid Unicode escape".to_owned())
    }

    fn read_number(&mut self) -> Result<DecodedValue, String> {
        let start = self.offset;
        if self.peek_byte() == Some(b'-') {
            self.offset += 1;
        }
        match self.peek_byte() {
            Some(b'0') => {
                self.offset += 1;
                if matches!(self.peek_byte(), Some(b'0'..=b'9')) {
                    return Err("A number cannot contain a leading zero".to_owned());
                }
            }
            Some(b'1'..=b'9') => self.read_digits(),
            _ => return Err("Expected a digit".to_owned()),
        }
        if self.peek_byte() == Some(b'.') {
            self.offset += 1;
            if !matches!(self.peek_byte(), Some(b'0'..=b'9')) {
                return Err("Expected a digit after decimal point".to_owned());
            }
            self.read_digits();
        }
        if matches!(self.peek_byte(), Some(b'e' | b'E')) {
            self.offset += 1;
            if matches!(self.peek_byte(), Some(b'+' | b'-')) {
                self.offset += 1;
            }
            if !matches!(self.peek_byte(), Some(b'0'..=b'9')) {
                return Err("Expected an exponent digit".to_owned());
            }
            self.read_digits();
        }
        let number = self.source[start..self.offset]
            .parse::<f64>()
            .map_err(|_| "Invalid number".to_owned())?;
        Ok(DecodedValue::Number(number))
    }

    fn read_digits(&mut self) {
        while matches!(self.peek_byte(), Some(b'0'..=b'9')) {
            self.offset += 1;
        }
    }

    fn read_keyword(&mut self, keyword: &str) -> Result<(), String> {
        if self.source[self.offset..].starts_with(keyword) {
            self.offset += keyword.len();
            Ok(())
        } else {
            Err(format!("Expected '{keyword}'"))
        }
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.peek_byte(), Some(b' ' | b'\n' | b'\r' | b'\t')) {
            self.offset += 1;
        }
    }

    fn peek_byte(&self) -> Option<u8> {
        self.source.as_bytes().get(self.offset).copied()
    }
}

fn materialize<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: DecodedValue,
) -> Result<v8::Local<'s, v8::Value>, String> {
    match value {
        DecodedValue::Null => Ok(v8::null(scope).into()),
        DecodedValue::Boolean(value) => Ok(v8::Boolean::new(scope, value).into()),
        DecodedValue::Number(value) => Ok(v8::Number::new(scope, value).into()),
        DecodedValue::Text(value) => Ok(crate::webidl::string(scope, &value)?.into()),
        DecodedValue::Sequence(values) => {
            let array = v8::Array::new(scope, values.len() as i32);
            for (index, value) in values.into_iter().enumerate() {
                let value = materialize(scope, value)?;
                if array.set_index(scope, index as u32, value) != Some(true) {
                    return Err("Cannot create decoded sequence".to_owned());
                }
            }
            Ok(array.into())
        }
        DecodedValue::Record(members) => {
            let object = v8::Object::new(scope);
            for (name, value) in members {
                let key = crate::webidl::string(scope, &name)?;
                let value = materialize(scope, value)?;
                if object.define_own_property(scope, key.into(), value, v8::PropertyAttribute::NONE)
                    != Some(true)
                {
                    return Err("Cannot create decoded record".to_owned());
                }
            }
            Ok(object.into())
        }
    }
}

pub(crate) fn encode(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> Result<String, String> {
    let mut active_objects = HashSet::new();
    encode_value(scope, value, &mut active_objects, false)?
        .ok_or_else(|| "The value cannot be represented as structured text".to_owned())
}

fn encode_value(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
    active_objects: &mut HashSet<i32>,
    sequence_element: bool,
) -> Result<Option<String>, String> {
    if value.is_null() {
        return Ok(Some("null".to_owned()));
    }
    if value.is_boolean() {
        return Ok(Some(if value.boolean_value(scope) {
            "true".to_owned()
        } else {
            "false".to_owned()
        }));
    }
    if value.is_number() {
        let number = value.number_value(scope).unwrap_or(f64::NAN);
        return Ok(Some(if number.is_finite() {
            if number == 0.0 {
                "0".to_owned()
            } else {
                number.to_string()
            }
        } else {
            "null".to_owned()
        }));
    }
    if value.is_string() || value.is_string_object() {
        return Ok(Some(quote_text(&crate::webidl::value_to_string(
            scope, value,
        ))));
    }
    if value.is_big_int() {
        return Err("BigInt values cannot be encoded".to_owned());
    }
    if value.is_undefined() || value.is_function() || value.is_symbol() {
        return Ok(sequence_element.then(|| "null".to_owned()));
    }
    let object = v8::Local::<v8::Object>::try_from(value)
        .map_err(|_| "The value cannot be encoded".to_owned())?;
    let identity = object.get_identity_hash().get();
    if !active_objects.insert(identity) {
        return Err("Cyclic object value".to_owned());
    }
    let encoded = if value.is_array() {
        let array = v8::Local::<v8::Array>::try_from(value)
            .map_err(|_| "Cannot inspect sequence".to_owned())?;
        let mut output = String::from("[");
        for index in 0..array.length() {
            if index > 0 {
                output.push(',');
            }
            let item = array
                .get_index(scope, index)
                .unwrap_or_else(|| v8::undefined(scope).into());
            let item = encode_value(scope, item, active_objects, true)?
                .unwrap_or_else(|| "null".to_owned());
            output.push_str(&item);
        }
        output.push(']');
        output
    } else {
        let names = object
            .get_own_property_names(
                scope,
                v8::GetPropertyNamesArgs {
                    mode: v8::KeyCollectionMode::OwnOnly,
                    property_filter: v8::PropertyFilter::ONLY_ENUMERABLE,
                    index_filter: v8::IndexFilter::IncludeIndices,
                    key_conversion: v8::KeyConversionMode::ConvertToString,
                },
            )
            .ok_or_else(|| "Cannot enumerate object members".to_owned())?;
        let mut output = String::from("{");
        let mut emitted = 0_u32;
        for index in 0..names.length() {
            let Some(key) = names.get_index(scope, index) else {
                continue;
            };
            let Some(member) = object.get(scope, key) else {
                continue;
            };
            let Some(member) = encode_value(scope, member, active_objects, false)? else {
                continue;
            };
            if emitted > 0 {
                output.push(',');
            }
            output.push_str(&quote_text(&crate::webidl::value_to_string(scope, key)));
            output.push(':');
            output.push_str(&member);
            emitted += 1;
        }
        output.push('}');
        output
    };
    active_objects.remove(&identity);
    Ok(Some(encoded))
}

fn quote_text(text: &str) -> String {
    let mut output = String::with_capacity(text.len() + 2);
    output.push('"');
    for character in text.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\u{0008}' => output.push_str("\\b"),
            '\u{000c}' => output.push_str("\\f"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            value if value <= '\u{001f}' => {
                output.push_str(&format!("\\u{:04x}", value as u32));
            }
            value => output.push(value),
        }
    }
    output.push('"');
    output
}
