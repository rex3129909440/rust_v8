pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "substringData", 2, substring_data)
}

fn substring_data(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(data) = super::character_data::data_if_character(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let units: Vec<u16> = data.encode_utf16().collect();
    let offset = arguments.get(0).uint32_value(scope).unwrap_or(0) as usize;
    if offset > units.len() {
        super::node::throw_dom_exception(
            scope,
            "IndexSizeError",
            "The offset is larger than the data length",
        );
        return;
    }
    let count = arguments.get(1).uint32_value(scope).unwrap_or(0) as usize;
    let end = offset.saturating_add(count).min(units.len());
    let value = String::from_utf16_lossy(&units[offset..end]);
    if let Some(value) = v8::String::new(scope, &value) {
        result.set(value.into());
    }
}
