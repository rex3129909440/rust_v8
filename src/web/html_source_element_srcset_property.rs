use super::html_source_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "srcset", get_string, set_string)
}

fn get_string(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let n = "srcset".to_owned();
    let Some(x) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(v) = v8::String::new(scope, x.strings.get(&n).map_or("", String::as_str)) {
        r.set(v.into())
    }
}

fn set_string(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let n = "srcset".to_owned();
    let v = crate::webidl::value_to_string(scope, a.get(0));
    if let Some(x) = scope
        .get_slot_mut::<HtmlSourceElementStore>()
        .and_then(|s| s.records.get_mut(&a.this().get_identity_hash().get()))
    {
        x.strings.insert(n, v);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
