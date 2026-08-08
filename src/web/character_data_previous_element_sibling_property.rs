pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "previousElementSibling",
        get_previous_element_sibling,
    )
}

fn get_previous_element_sibling(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if super::character_data::data_if_character(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    match super::character_data::element_sibling(scope, arguments.this(), false) {
        Some(element) => result.set(element.into()),
        None => result.set(v8::null(scope).into()),
    }
}
