use super::html_form_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "length", get_length)
}

fn get_length(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let controls = collect_controls(scope, arguments.this());
    let length = controls.len();
    let collection = v8::Local::new(scope, &record.elements);
    super::html_form_controls_collection::replace(scope, collection, controls);
    result.set(v8::Integer::new_from_unsigned(scope, length as u32).into());
}
