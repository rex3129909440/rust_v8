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
    if !super::element_method_support::ensure(scope, arguments.this()) {
        return;
    }
    let element = arguments.this();
    let rect = if super::inline_text_layout::uses_inline_fragment_geometry(scope, element) {
        let rects = super::inline_text_layout::inline_element_rects(scope, element);
        if rects.is_empty() {
            super::element_layout::bounding_rect(scope, element)
        } else {
            super::range_geometry::bounding_rect(&rects)
        }
    } else {
        super::element_layout::bounding_rect(scope, element)
    };
    match super::dom_rect::create(scope, rect) {
        Ok(rect) => result.set(rect.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
