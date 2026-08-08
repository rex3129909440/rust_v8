use super::html_link_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "rel", get_rel, set_rel)
}

fn get_rel(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if record(scope, a.this()).is_some() {
        let value = super::element::reflected_string(scope, a.this(), "rel").unwrap_or_default();
        if let Some(value) = v8::String::new(scope, &value) {
            r.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_rel(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, a.get(0));
    if let Some(record) = record(scope, a.this()) {
        let list = v8::Local::new(scope, &record.rel_list);
        let _ = super::dom_token_list::set_string_value(scope, list, &value);
        super::element::set_reflected_string(scope, a.this(), "rel", value);
        refresh_connected(scope, a.this());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
