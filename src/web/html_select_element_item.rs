use super::html_select_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "item", 1, item)
}

fn item(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if record(scope, a.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let i = a.get(0).uint32_value(scope).unwrap_or(u32::MAX) as usize;
    if let Some(o) = options_snapshot(scope, a.this()).get(i) {
        r.set((*o).into())
    } else {
        r.set(v8::null(scope).into())
    }
}
