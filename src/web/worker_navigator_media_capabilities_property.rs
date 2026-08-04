pub(crate) fn define(
    s: &mut v8::PinScope<'_, '_>,
    p: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(s, p, "mediaCapabilities", get)
}
fn get(s: &mut v8::PinScope<'_, '_>, a: v8::FunctionCallbackArguments<'_>, r: v8::ReturnValue<'_>) {
    super::worker_navigator::get_media_capabilities(s, a, r);
}
