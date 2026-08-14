pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "extend", 1, extend)
}
fn extend(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(current) = super::selection::record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let Ok(node) = v8::Local::<v8::Object>::try_from(a.get(0)) else {
        crate::webidl::throw_type_error(scope, "extend requires a Node");
        return;
    };
    let offset = a.get(1).uint32_value(scope).unwrap_or(0);
    if !super::selection::valid_offset(scope, node, offset) {
        super::node::throw_dom_exception(scope, "IndexSizeError", "The offset is out of bounds");
        return;
    }
    let Some(anchor) = current.anchor else {
        super::node::throw_dom_exception(
            scope,
            "InvalidStateError",
            "Failed to execute 'extend' on 'Selection': This Selection object doesn't have any Ranges.",
        );
        return;
    };
    let anchor_local = v8::Local::new(scope, &anchor);
    let range =
        super::selection::selection_range(scope, anchor_local, current.anchor_offset, node, offset);
    let direction = super::selection::direction_between(
        scope,
        anchor_local,
        current.anchor_offset,
        node,
        offset,
    );
    let node = v8::Global::new(scope, node);
    super::selection::update(scope, a.this(), |v| {
        v.focus = Some(node);
        v.focus_offset = offset;
        v.ranges = range.into_iter().collect();
        v.direction = direction;
    })
}
