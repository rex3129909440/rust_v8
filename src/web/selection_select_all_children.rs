pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(
        scope,
        prototype,
        "selectAllChildren",
        1,
        select_all_children,
    )
}
fn select_all_children(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if super::selection::record(scope, a.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let Ok(node) = v8::Local::<v8::Object>::try_from(a.get(0)) else {
        crate::webidl::throw_type_error(scope, "selectAllChildren requires a Node");
        return;
    };
    if super::node::record(scope, node).is_some_and(|record| record.node_type == 10) {
        let node_name = super::node::record(scope, node)
            .map(|record| record.node_name)
            .unwrap_or_default();
        super::node::throw_dom_exception(
            scope,
            "InvalidNodeTypeError",
            &format!(
                "Failed to execute 'selectAllChildren' on 'Selection': The node provided is of type '{node_name}'."
            ),
        );
        return;
    }
    let end = super::node::children(scope, node).len() as u32;
    let range = super::selection::selection_range(scope, node, 0, node, end);
    let one = v8::Global::new(scope, node);
    let two = v8::Global::new(scope, node);
    super::selection::update(scope, a.this(), |v| {
        v.anchor = Some(one);
        v.focus = Some(two);
        v.anchor_offset = 0;
        v.focus_offset = end;
        v.ranges = range.into_iter().collect();
        v.direction = "forward".to_owned();
    })
}
