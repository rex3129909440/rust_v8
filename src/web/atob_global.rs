pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let function =
        crate::webidl::create_function(scope, "atob", 1, v8::ConstructorBehavior::Throw, atob)?;
    let global = scope.get_current_context().global(scope);
    let key = crate::webidl::string(scope, "atob")?;
    if global.define_own_property(
        scope,
        key.into(),
        function.into(),
        v8::PropertyAttribute::NONE,
    ) == Some(true)
    {
        Ok(())
    } else {
        Err("cannot define window.atob".to_owned())
    }
}
fn atob(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'atob' on 'Window': 1 argument required, but only 0 present.",
        );
        return;
    }
    let source = crate::webidl::value_to_string(scope, arguments.get(0));
    let compact = source
        .bytes()
        .filter(|byte| !byte.is_ascii_whitespace())
        .collect::<Vec<_>>();
    let Some(bytes) = decode_base64(&compact) else {
        throw_invalid_character(
            scope,
            "Failed to execute 'atob' on 'Window': The string to be decoded is not correctly encoded.",
        );
        return;
    };
    let value = bytes.into_iter().map(char::from).collect::<String>();
    if let Some(value) = v8::String::new(scope, &value) {
        result.set(value.into());
    }
}
fn decode_base64(input: &[u8]) -> Option<Vec<u8>> {
    if input.len() % 4 == 1 {
        return None;
    }
    let mut padded = input.to_vec();
    while padded.len() % 4 != 0 {
        padded.push(b'=');
    }
    let mut output = Vec::with_capacity(padded.len() / 4 * 3);
    for chunk in padded.chunks_exact(4) {
        let first = base64_value(chunk[0])?;
        let second = base64_value(chunk[1])?;
        let third = if chunk[2] == b'=' {
            64
        } else {
            base64_value(chunk[2])?
        };
        let fourth = if chunk[3] == b'=' {
            64
        } else {
            base64_value(chunk[3])?
        };
        if third == 64 && fourth != 64 {
            return None;
        }
        output.push((first << 2) | (second >> 4));
        if third != 64 {
            output.push((second << 4) | (third >> 2));
        }
        if fourth != 64 {
            output.push((third << 6) | fourth);
        }
    }
    Some(output)
}
fn base64_value(value: u8) -> Option<u8> {
    match value {
        b'A'..=b'Z' => Some(value - b'A'),
        b'a'..=b'z' => Some(value - b'a' + 26),
        b'0'..=b'9' => Some(value - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}
fn throw_invalid_character(scope: &mut v8::PinScope<'_, '_>, message: &str) {
    match super::dom_exception::create(
        scope,
        message.to_owned(),
        "InvalidCharacterError".to_owned(),
    ) {
        Ok(exception) => {
            scope.throw_exception(exception.into());
        }
        Err(_) => crate::webidl::throw_type_error(scope, message),
    }
}
