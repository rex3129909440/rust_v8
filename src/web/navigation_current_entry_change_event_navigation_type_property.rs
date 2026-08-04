use super::navigation_current_entry_change_event::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "navigationType", get_navigation_type)
}

fn get_navigation_type(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    match record(scope, a.this()) {
        Some(v) => match v.navigation_type {
            Some(value) => {
                if let Some(s) = v8::String::new(scope, &value) {
                    r.set(s.into())
                }
            }
            None => r.set(v8::null(scope).into()),
        },
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}
