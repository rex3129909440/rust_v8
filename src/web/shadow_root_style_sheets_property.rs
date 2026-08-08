pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "styleSheets", get_style_sheets)
}
fn get_style_sheets(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if super::shadow_root::record(scope, a.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    match super::style_sheet_list::create(scope, Vec::new()) {
        Ok(v) => r.set(v.into()),
        Err(m) => crate::webidl::throw_type_error(scope, &m),
    }
}
