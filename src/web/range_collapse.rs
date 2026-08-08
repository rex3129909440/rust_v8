pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "collapse", 0, collapse)
}
fn collapse(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let to_start = arguments.get(0).boolean_value(scope);
    let Some(record) = super::range::record_or_throw(scope, arguments.this()) else {
        return;
    };
    let (container, offset) = if to_start {
        (record.start_container, record.start_offset)
    } else {
        (record.end_container, record.end_offset)
    };
    super::abstract_range::update(scope, arguments.this(), |range| {
        range.start_container = container.clone();
        range.start_offset = offset;
        range.end_container = container;
        range.end_offset = offset;
    });
}
