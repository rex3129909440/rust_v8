use super::html_area_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "rel", get_rel, set_rel)
}

fn get_rel(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(x) = record(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    let o = v8::Local::new(s, &x.rel_list);
    let v = super::dom_token_list::string_value(s, o).unwrap_or_default();
    if let Some(v) = v8::String::new(s, &v) {
        r.set(v.into())
    }
}

fn set_rel(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = crate::webidl::value_to_string(s, a.get(0));
    if let Some(x) = record(s, a.this()) {
        let o = v8::Local::new(s, &x.rel_list);
        super::dom_token_list::set_string_value(s, o, &v);
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
