pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "comparePoint", 2, compare_point)
}
fn compare_point(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = super::range::record_or_throw(scope, arguments.this()) else {
        return;
    };
    let Some((node, offset)) = super::range::boundary_arguments(scope, &arguments) else {
        return;
    };
    let node = v8::Local::new(scope, &node);
    let start = v8::Local::new(scope, &record.start_container);
    let end = v8::Local::new(scope, &record.end_container);
    let Some(before) =
        super::range::compare_boundaries(scope, node, offset, start, record.start_offset)
    else {
        super::node::throw_dom_exception(
            scope,
            "WrongDocumentError",
            "The point and range have different roots",
        );
        return;
    };
    let Some(after) = super::range::compare_boundaries(scope, node, offset, end, record.end_offset)
    else {
        super::node::throw_dom_exception(
            scope,
            "WrongDocumentError",
            "The point and range have different roots",
        );
        return;
    };
    let answer = if before < 0 {
        -1
    } else if after > 0 {
        1
    } else {
        0
    };
    result.set(v8::Integer::new(scope, answer).into());
}
