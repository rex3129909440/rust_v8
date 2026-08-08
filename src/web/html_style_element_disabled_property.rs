use super::html_style_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "disabled", get_disabled, set_disabled)
}

fn get_disabled(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(x) = record(scope, a.this()) {
        let disabled = x
            .sheet
            .as_ref()
            .map(|sheet| {
                let sheet = v8::Local::new(scope, sheet);
                super::style_sheet::is_disabled(scope, sheet)
            })
            .unwrap_or(x.disabled);
        r.set(v8::Boolean::new(scope, disabled).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}

fn set_disabled(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = a.get(0).boolean_value(scope);
    if let Some(x) = scope
        .get_slot_mut::<HtmlStyleElementStore>()
        .and_then(|s| s.records.get_mut(&a.this().get_identity_hash().get()))
    {
        x.disabled = v;
        if let Some(sheet) = x.sheet.clone() {
            let sheet = v8::Local::new(scope, &sheet);
            super::style_sheet::set_disabled_value(scope, sheet, v);
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
