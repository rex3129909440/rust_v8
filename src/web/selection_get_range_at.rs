pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "getRangeAt", 1, get_range_at)
}
fn get_range_at(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(v) = super::selection::record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let i = a.get(0).uint32_value(scope).unwrap_or(u32::MAX) as usize;
    if let Some(range) = v.ranges.get(i) {
        r.set(v8::Local::new(scope, range).into())
    } else {
        super::node::throw_dom_exception(
            scope,
            "IndexSizeError",
            &format!("Failed to execute 'getRangeAt' on 'Selection': {i} is not a valid index."),
        )
    }
}
