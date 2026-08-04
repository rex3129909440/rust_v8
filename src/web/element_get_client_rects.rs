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
    if !super::element_method_support::ensure(scope, arguments.this()) {
        return;
    }
    let layout = super::element_layout::compute(scope, arguments.this());
    if !layout.rendered {
        match super::dom_rect_list::create(scope, Vec::new()) {
            Ok(list) => result.set(list.into()),
            Err(message) => crate::webidl::throw_type_error(scope, &message),
        }
        return;
    }
    let rect = layout.rect();
    let value = match super::dom_rect::create(scope, rect) {
        Ok(value) => value,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    match super::dom_rect_list::create(scope, vec![value]) {
        Ok(list) => result.set(list.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
