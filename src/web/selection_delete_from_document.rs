pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(
        scope,
        prototype,
        "deleteFromDocument",
        0,
        delete_from_document,
    )
}
fn delete_from_document(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(v) = super::selection::record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let Some(range) = v.ranges.first() else {
        return;
    };
    let range = v8::Local::new(scope, range);
    if super::range_contents::delete_contents(scope, range).is_err() {
        return;
    }
    let Some(boundary) = super::abstract_range::record(scope, range) else {
        return;
    };
    let anchor = boundary.start_container.clone();
    let focus = boundary.start_container;
    let offset = boundary.start_offset;
    super::selection::update(scope, a.this(), |x| {
        x.anchor = Some(anchor);
        x.focus = Some(focus);
        x.anchor_offset = offset;
        x.focus_offset = offset;
        x.direction = "none".to_owned();
    })
}
