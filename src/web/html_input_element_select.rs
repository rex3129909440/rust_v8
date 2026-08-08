use super::html_input_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "select", 0, select)
}

fn select(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(current) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if supports_selection(&current.input_type) {
        update(scope, a.this(), |x| {
            x.selection_start = 0;
            x.selection_end = x.value.chars().count() as u32;
            x.selection_direction = "forward".to_owned();
        });
    }
}
