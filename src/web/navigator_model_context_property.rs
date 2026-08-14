pub(crate) fn define(
    s: &mut v8::PinScope<'_, '_>,
    p: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(s, p, "modelContext", get)
}
fn get(s: &mut v8::PinScope<'_, '_>, a: v8::FunctionCallbackArguments<'_>, r: v8::ReturnValue<'_>) {
    super::navigator::get_android_object(
        s,
        a,
        r,
        super::navigator::AndroidObjectProperty::ModelContext,
    )
}
