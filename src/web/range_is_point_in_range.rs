pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "isPointInRange", 2, is_point_in_range)
}
fn is_point_in_range(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = super::range::record_or_throw(scope, arguments.this()) else {
        return;
    };
    let Some((node, offset)) =
        super::range::boundary_arguments(scope, &arguments, "isPointInRange")
    else {
        return;
    };
    let node = v8::Local::new(scope, &node);
    let start = v8::Local::new(scope, &record.start_container);
    let end = v8::Local::new(scope, &record.end_container);
    let Some(start_order) =
        super::range::compare_boundaries(scope, node, offset, start, record.start_offset)
    else {
        result.set(v8::Boolean::new(scope, false).into());
        return;
    };
    let Some(end_order) =
        super::range::compare_boundaries(scope, node, offset, end, record.end_offset)
    else {
        result.set(v8::Boolean::new(scope, false).into());
        return;
    };
    let after_start = start_order >= 0;
    let before_end = end_order <= 0;
    result.set(v8::Boolean::new(scope, after_start && before_end).into());
}
