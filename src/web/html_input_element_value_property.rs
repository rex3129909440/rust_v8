use super::html_input_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "value", get_value, set_value)
}

fn get_value(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.value);
}

fn set_value(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(s, a.get(0));
    update(s, a.this(), |x| {
        if x.input_type == "file" && !value.is_empty() {
            return;
        }
        x.value = sanitize_value(&x.input_type, value);
        x.value_dirty = true;
        let length = x.value.chars().count() as u32;
        x.selection_start = length;
        x.selection_end = length;
        x.selection_direction = "forward".to_owned();
    });
}
