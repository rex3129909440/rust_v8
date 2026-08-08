use super::svg_animation_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "systemLanguage", get_system_language)
}

fn get_system_language(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_list(scope, &record.system_language, result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
