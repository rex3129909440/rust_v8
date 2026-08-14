pub(crate) fn define(
    s: &mut v8::PinScope<'_, '_>,
    p: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(
        s,
        p,
        "fullscreenElement",
        get_fullscreen_element,
        set_fullscreen_element,
    )
}
fn set_fullscreen_element(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let _ = super::document_property_support::ensure(s, a.this());
}
fn get_fullscreen_element(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    super::document_property_support::get_nullable_stored(s, a, r, "fullscreenElement")
}
