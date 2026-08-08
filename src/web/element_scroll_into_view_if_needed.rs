pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(
        scope,
        prototype,
        "scrollIntoViewIfNeeded",
        0,
        scroll_into_view_if_needed,
    )
}

fn scroll_into_view_if_needed(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if !super::element_method_support::ensure(scope, arguments.this()) {
        return;
    }
    let center = arguments.get(0).is_undefined() || arguments.get(0).boolean_value(scope);
    let alignment = if center {
        super::element_scroll_into_view::Alignment::Center
    } else {
        super::element_scroll_into_view::Alignment::Nearest
    };
    super::element_scroll_into_view::scroll_element_into_view(
        scope,
        arguments.this(),
        alignment,
        alignment,
        center,
        false,
    );
}
