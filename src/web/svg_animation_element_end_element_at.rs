use super::svg_animation_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "endElementAt", 1, end_element_at)
}

fn end_element_at(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let offset = arguments.get(0).number_value(scope).unwrap_or(0.0);
    let handler = record(scope, arguments.this()).and_then(|record| record.onend);
    update(scope, arguments.this(), |record| {
        record.duration = (record.current_time + offset - record.start_time).max(0.0);
        record.active = false;
    });
    fire(scope, arguments.this(), "end", handler);
    result.set(v8::Boolean::new(scope, true).into());
}
