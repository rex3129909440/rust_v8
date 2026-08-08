pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "containsNode", 1, contains_node)
}
fn contains_node(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(v) = super::selection::record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let Ok(node) = v8::Local::<v8::Object>::try_from(a.get(0)) else {
        crate::webidl::throw_type_error(scope, "containsNode requires a Node");
        return;
    };
    if super::node::record(scope, node).is_none() {
        crate::webidl::throw_type_error(scope, "containsNode requires a Node");
        return;
    }
    let partial = a.get(1).boolean_value(scope);
    let value = (|| {
        let range = v.ranges.first()?;
        let range = super::abstract_range::record(scope, v8::Local::new(scope, range))?;
        let parent = super::node::parent(scope, node)?;
        let index = super::node::children(scope, parent)
            .iter()
            .position(|child| child.strict_equals(node.into()))? as u32;
        let start = v8::Local::new(scope, &range.start_container);
        let end = v8::Local::new(scope, &range.end_container);
        let start_to_node_start =
            super::range::compare_boundaries(scope, start, range.start_offset, parent, index)?;
        let start_to_node_end =
            super::range::compare_boundaries(scope, start, range.start_offset, parent, index + 1)?;
        let node_start_to_end =
            super::range::compare_boundaries(scope, parent, index, end, range.end_offset)?;
        let node_end_to_end =
            super::range::compare_boundaries(scope, parent, index + 1, end, range.end_offset)?;
        Some(if partial {
            start_to_node_end < 0 && node_start_to_end < 0
        } else {
            start_to_node_start <= 0 && node_end_to_end <= 0
        })
    })()
    .unwrap_or(false);
    r.set(v8::Boolean::new(scope, value).into())
}
