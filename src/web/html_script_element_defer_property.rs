use super::html_script_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "defer", get_boolean, set_boolean)
}

fn get_boolean(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let n = "defer".to_owned();
    let Some(x) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    r.set(v8::Boolean::new(scope, x.booleans.get(&n).copied().unwrap_or(false)).into())
}

fn set_boolean(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let n = "defer".to_owned();
    let v = a.get(0).boolean_value(scope);
    if let Some(x) = scope
        .get_slot_mut::<HtmlScriptElementStore>()
        .and_then(|s| s.records.get_mut(&a.this().get_identity_hash().get()))
    {
        x.booleans.insert(n, v);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
