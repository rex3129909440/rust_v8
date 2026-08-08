pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let function =
        crate::webidl::create_function(scope, "btoa", 1, v8::ConstructorBehavior::Throw, btoa)?;
    let global = scope.get_current_context().global(scope);
    let key = crate::webidl::string(scope, "btoa")?;
    if global.define_own_property(
        scope,
        key.into(),
        function.into(),
        v8::PropertyAttribute::NONE,
    ) == Some(true)
    {
        Ok(())
    } else {
        Err("cannot define window.btoa".to_owned())
    }
}
fn btoa(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'btoa' on 'Window': 1 argument required, but only 0 present.",
        );
        return;
    }
    let source = crate::webidl::value_to_string(scope, arguments.get(0));
    let mut bytes = Vec::with_capacity(source.len());
    for character in source.chars() {
        let value = character as u32;
        if value > 255 {
            throw_invalid_character(
                scope,
                "Failed to execute 'btoa' on 'Window': The string to be encoded contains characters outside of the Latin1 range.",
            );
            return;
        }
        bytes.push(value as u8);
    }
    let encoded = encode_base64(&bytes);
    if let Some(value) = v8::String::new(scope, &encoded) {
        result.set(value.into());
    }
}
fn encode_base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut output = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let first = chunk[0];
        let second = chunk.get(1).copied();
        let third = chunk.get(2).copied();
        output.push(ALPHABET[(first >> 2) as usize] as char);
        output.push(ALPHABET[(((first & 3) << 4) | second.unwrap_or(0) >> 4) as usize] as char);
        if let Some(second) = second {
            output
                .push(ALPHABET[(((second & 15) << 2) | third.unwrap_or(0) >> 6) as usize] as char);
        } else {
            output.push('=');
        }
        if let Some(third) = third {
            output.push(ALPHABET[(third & 63) as usize] as char);
        } else {
            output.push('=');
        }
    }
    output
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
