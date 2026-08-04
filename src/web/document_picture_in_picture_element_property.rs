pub(crate) fn define(
    s: &mut v8::PinScope<'_, '_>,
    p: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(
        s,
        p,
        "pictureInPictureElement",
        get_picture_in_picture_element,
    )
}
fn get_picture_in_picture_element(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    super::document_property_support::get_nullable_stored(s, a, r, "pictureInPictureElement")
}
