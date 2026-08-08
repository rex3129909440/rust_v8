pub(crate) fn define(
    s: &mut v8::PinScope<'_, '_>,
    p: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(
        s,
        p,
        "ariaActiveDescendantElement",
        get_element,
        set_element,
    )
}
fn get_element(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    super::element_aria_reflection::get_element(
        s,
        a,
        r,
        "ariaActiveDescendantElement",
        "aria-activedescendant",
    )
}
fn set_element(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    super::element_aria_reflection::set_element(
        s,
        a,
        "ariaActiveDescendantElement",
        "aria-activedescendant",
    )
}
