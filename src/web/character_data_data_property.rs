pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "data", get_data, set_data)
}

fn get_data(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(data) = super::character_data::data_if_character(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(value) = v8::String::new(scope, &data) {
        result.set(value.into());
    }
}

fn set_data(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if super::character_data::data_if_character(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let Some(data) = super::character_data::string_argument(
        scope,
        arguments.get(0),
        "Failed to set the 'data' property on 'CharacterData'",
    ) else {
        return;
    };
    if !super::character_data::set_data_if_character(scope, arguments.this(), data) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
