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
    let Ok(node) = v8::Local::<v8::Object>::try_from(a.get(0)) else {
        crate::webidl::throw_type_error(scope, "selectAllChildren requires a Node");
        return;
    };
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
