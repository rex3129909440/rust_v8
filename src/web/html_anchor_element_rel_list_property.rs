use super::html_anchor_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "relList", get_rel_list, set_rel_list)
}

fn get_rel_list(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(x) = record(s, a.this()) {
        r.set(v8::Local::new(s, &x.rel_list).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}

fn set_rel_list(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, a.get(0));
    if let Some(record) = record(scope, a.this()) {
        let rel_list = v8::Local::new(scope, &record.rel_list);
        super::dom_token_list::set_string_value(scope, rel_list, &value);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
