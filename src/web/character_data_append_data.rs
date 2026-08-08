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
    let Some(mut data) = super::character_data::data_if_character(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    data.push_str(&crate::webidl::value_to_string(scope, arguments.get(0)));
    let _ = super::character_data::set_data_if_character(scope, arguments.this(), data);
}
