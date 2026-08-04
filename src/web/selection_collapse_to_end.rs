pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "collapseToEnd", 0, collapse_to_end)
}
fn collapse_to_end(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(v) = super::selection::record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let Some(range) = v.ranges.first() else {
        super::node::throw_dom_exception(scope, "InvalidStateError", "The Selection has no Range");
        return;
    };
    let range = v8::Local::new(scope, range);
    let Some(boundary) = super::abstract_range::record(scope, range) else {
        return;
    };
    let node = v8::Local::new(scope, &boundary.end_container);
    let anchor = v8::Global::new(scope, node);
    let focus = v8::Global::new(scope, node);
    let offset = boundary.end_offset;
    let collapsed = super::selection::selection_range(scope, node, offset, node, offset);
    super::selection::update(scope, a.this(), |x| {
        x.anchor = Some(anchor);
        x.focus = Some(focus);
        x.anchor_offset = offset;
        x.focus_offset = offset;
        x.ranges = collapsed.into_iter().collect();
        x.direction = "none".to_owned();
    })
}
