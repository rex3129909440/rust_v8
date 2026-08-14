pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "appendData", 1, append_data)
}

fn append_data(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(data) = super::character_data::data_if_character(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if !super::character_data::require_arguments(scope, &arguments, "appendData", 1) {
        return;
    }
    let Some(value) = super::character_data::string_argument(
        scope,
        arguments.get(0),
        "Failed to execute 'appendData' on 'CharacterData'",
    ) else {
        return;
    };
    let length = data.encode_utf16().count() as u32;
    let _ = super::character_data::replace_data_units(scope, arguments.this(), length, 0, &value);
}
