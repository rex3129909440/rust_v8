use super::html_style_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "sheet", get_sheet)
}

fn get_sheet(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(x) = record(scope, a.this()) {
        if let Some(sheet) = x.sheet {
            r.set(v8::Local::new(scope, &sheet).into())
        } else {
            r.set(v8::null(scope).into())
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
