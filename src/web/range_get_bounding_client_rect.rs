pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(
        scope,
        prototype,
        "getBoundingClientRect",
        0,
        get_bounding_client_rect,
    )
}
fn get_bounding_client_rect(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = super::range::record_or_throw(scope, arguments.this()) else {
        return;
    };
    let rects = super::range_geometry::client_rects(scope, &record);
    let rect = super::range_geometry::bounding_rect(&rects);
    match super::dom_rect::create(scope, rect) {
        Ok(rect) => result.set(rect.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
