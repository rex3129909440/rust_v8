pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "removeRange", 1, remove_range)
}
fn remove_range(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(current) = super::selection::record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let Ok(range) = v8::Local::<v8::Object>::try_from(a.get(0)) else {
        return;
    };
    let found = current
        .ranges
        .iter()
        .any(|value| v8::Local::new(scope, value).strict_equals(range.into()));
    if !found {
        super::node::throw_dom_exception(
            scope,
            "NotFoundError",
            "Failed to execute 'removeRange' on 'Selection': Range not found.",
        );
        return;
    }
    super::selection::update(scope, a.this(), |v| {
        v.ranges.clear();
        v.anchor = None;
        v.focus = None;
        v.anchor_offset = 0;
        v.focus_offset = 0;
        v.direction = "none".to_owned();
    })
}
