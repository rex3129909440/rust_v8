use super::svg_animation_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "endElement", 0, end_element)
}

fn end_element(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let handler = record(scope, arguments.this()).and_then(|record| record.onend);
    update(scope, arguments.this(), |record| record.active = false);
    fire(scope, arguments.this(), "end", handler);
    result.set(v8::Boolean::new(scope, true).into());
}
