use super::html_select_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "namedItem", 1, named_item)
}

fn named_item(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if record(scope, a.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let name = crate::webidl::value_to_string(scope, a.get(0));
    for o in options_snapshot(scope, a.this()) {
        if super::element::record(scope, o).is_some_and(|e| {
            e.attributes.iter().any(|(n, v)| {
                (n.eq_ignore_ascii_case("id") || n.eq_ignore_ascii_case("name")) && v == &name
            })
        }) {
            r.set(o.into());
            return;
        }
    }
    r.set(v8::null(scope).into())
}
