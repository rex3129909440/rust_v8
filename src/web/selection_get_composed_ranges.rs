pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(
        scope,
        prototype,
        "getComposedRanges",
        0,
        get_composed_ranges,
    )
}
fn get_composed_ranges(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(v) = super::selection::record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let out = v8::Array::new(scope, v.ranges.len() as i32);
    for (i, range) in v.ranges.iter().enumerate() {
        let range = v8::Local::new(scope, range);
        let Some(record) = super::abstract_range::record(scope, range) else {
            continue;
        };
        let Ok(static_range) = super::static_range::create(scope, &record) else {
            continue;
        };
        let _ = out.set_index(scope, i as u32, static_range.into());
    }
    r.set(out.into())
}
