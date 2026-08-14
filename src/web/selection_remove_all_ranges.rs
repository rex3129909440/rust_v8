pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "removeAllRanges", 0, remove_all_ranges)
}
fn remove_all_ranges(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    super::selection::update(scope, arguments.this(), |selection| {
        selection.anchor = None;
        selection.focus = None;
        selection.anchor_offset = 0;
        selection.focus_offset = 0;
        selection.ranges.clear();
        selection.direction = "none".to_owned();
    });
}
