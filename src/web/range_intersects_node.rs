pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "intersectsNode", 1, intersects_node)
}
fn intersects_node(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(range) = super::range::record_or_throw(scope, arguments.this()) else {
        return;
    };
    let Ok(node) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        result.set(v8::Boolean::new(scope, false).into());
        return;
    };
    let start = v8::Local::new(scope, &range.start_container);
    let root_node = super::range::root(scope, start);
    let Some(node_start) = super::range::boundary_index(scope, root_node, node, 0) else {
        result.set(v8::Boolean::new(scope, false).into());
        return;
    };
    let node_end = node_start + super::range::text_length(scope, node);
    let range_start =
        super::range::boundary_index(scope, root_node, start, range.start_offset).unwrap_or(0);
    let end = v8::Local::new(scope, &range.end_container);
    let range_end = super::range::boundary_index(scope, root_node, end, range.end_offset)
        .unwrap_or(range_start);
    result.set(v8::Boolean::new(scope, node_end > range_start && node_start < range_end).into());
}
