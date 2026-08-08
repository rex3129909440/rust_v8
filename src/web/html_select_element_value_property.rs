use super::html_select_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "value", get_value, set_value)
}

fn get_value(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if record(scope, a.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let value = options_snapshot(scope, a.this())
        .into_iter()
        .find(|o| super::html_option_element::option_selected(scope, *o).unwrap_or(false))
        .and_then(|o| super::html_option_element::option_value(scope, o))
        .unwrap_or_default();
    if let Some(v) = v8::String::new(scope, &value) {
        r.set(v.into())
    }
}

fn set_value(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, a.get(0));
    if let Some(x) = scope
        .get_slot_mut::<HtmlSelectElementStore>()
        .and_then(|s| s.records.get_mut(&a.this().get_identity_hash().get()))
    {
        x.selection_explicit = true;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let mut found = false;
    for o in options_snapshot(scope, a.this()) {
        let matched = !found
            && super::html_option_element::option_value(scope, o).is_some_and(|v| v == value);
        let _ = super::html_option_element::set_option_selected(scope, o, matched);
        found |= matched;
    }
    refresh(scope, a.this())
}
