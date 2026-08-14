pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "collapse", 1, collapse)
}
fn collapse(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if a.get(0).is_null() {
        super::selection::update(scope, a.this(), |v| {
            v.anchor = None;
            v.focus = None;
            v.anchor_offset = 0;
            v.focus_offset = 0;
            v.ranges.clear();
            v.direction = "none".to_owned();
        });
        return;
    }
    let Ok(node) = v8::Local::<v8::Object>::try_from(a.get(0)) else {
        crate::webidl::throw_type_error(scope, "collapse requires a Node");
        return;
    };
    let offset = a.get(1).uint32_value(scope).unwrap_or(0);
    if !super::selection::valid_offset(scope, node, offset) {
        let length = super::range::boundary_length(scope, node).unwrap_or(0);
        super::node::throw_dom_exception(
            scope,
            "IndexSizeError",
            &format!(
                "Failed to execute 'collapse' on 'Selection': The offset {offset} is larger than the node's length ({length})."
            ),
        );
        return;
    }
    let range = super::selection::selection_range(scope, node, offset, node, offset);
    let one = v8::Global::new(scope, node);
    let two = v8::Global::new(scope, node);
    super::selection::update(scope, a.this(), |v| {
        v.anchor = Some(one);
        v.focus = Some(two);
        v.anchor_offset = offset;
        v.focus_offset = offset;
        v.ranges = range.into_iter().collect();
        v.direction = "none".to_owned();
    })
}
