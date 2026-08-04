pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(
        scope,
        prototype,
        "compareBoundaryPoints",
        2,
        compare_boundary_points,
    )
}
fn compare_boundary_points(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let how = arguments.get(0).int32_value(scope).unwrap_or(-1);
    let Ok(other) = v8::Local::<v8::Object>::try_from(arguments.get(1)) else {
        crate::webidl::throw_type_error(scope, "The second argument is not a Range");
        return;
    };
    let Some(left) = super::range::record_or_throw(scope, arguments.this()) else {
        return;
    };
    let Some(right) = super::abstract_range::record(scope, other) else {
        crate::webidl::throw_type_error(scope, "The second argument is not a Range");
        return;
    };
    let (a_node, a_offset, b_node, b_offset) = match how {
        super::range::START_TO_START => (
            &left.start_container,
            left.start_offset,
            &right.start_container,
            right.start_offset,
        ),
        super::range::START_TO_END => (
            &left.end_container,
            left.end_offset,
            &right.start_container,
            right.start_offset,
        ),
        super::range::END_TO_END => (
            &left.end_container,
            left.end_offset,
            &right.end_container,
            right.end_offset,
        ),
        super::range::END_TO_START => (
            &left.start_container,
            left.start_offset,
            &right.end_container,
            right.end_offset,
        ),
        _ => {
            super::node::throw_dom_exception(scope, "NotSupportedError", "Invalid comparison mode");
            return;
        }
    };
    let a_node = v8::Local::new(scope, a_node);
    let b_node = v8::Local::new(scope, b_node);
    let Some(ordering) =
        super::range::compare_boundaries(scope, a_node, a_offset, b_node, b_offset)
    else {
        super::node::throw_dom_exception(
            scope,
            "WrongDocumentError",
            "The ranges have different roots",
        );
        return;
    };
    result.set(v8::Integer::new(scope, ordering).into());
}
