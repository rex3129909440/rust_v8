pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "addRange", 1, add_range)
}
fn add_range(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Ok(range) = v8::Local::<v8::Object>::try_from(a.get(0)) else {
        crate::webidl::throw_type_error(scope, "addRange requires a Range");
        return;
    };
    let boundary = super::abstract_range::record(scope, range);
    if !boundary.as_ref().is_some_and(|record| record.live) {
        crate::webidl::throw_type_error(scope, "addRange requires a Range");
        return;
    }
    let range_global = v8::Global::new(scope, range);
    super::selection::update(scope, a.this(), |v| {
        if v.ranges.is_empty() {
            v.ranges.push(range_global);
        }
        if let Some(b) = boundary {
            v.anchor = Some(b.start_container);
            v.anchor_offset = b.start_offset;
            v.focus = Some(b.end_container);
            v.focus_offset = b.end_offset;
            v.direction = "forward".to_owned();
        }
    })
}
