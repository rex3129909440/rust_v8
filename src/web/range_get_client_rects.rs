pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "getClientRects", 0, get_client_rects)
}
fn get_client_rects(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = super::range::record_or_throw(scope, arguments.this()) else {
        return;
    };
    let rects = super::range_geometry::client_rects(scope, &record);
    let mut values = Vec::with_capacity(rects.len());
    for rect in rects {
        let Ok(rect) = super::dom_rect::create(scope, rect) else {
            crate::webidl::throw_type_error(scope, "Cannot create Range client rectangle");
            return;
        };
        values.push(rect);
    }
    match super::dom_rect_list::create(scope, values) {
        Ok(list) => result.set(list.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
