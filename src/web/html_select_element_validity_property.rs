use super::html_select_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "validity", get_validity)
}

fn get_validity(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(x) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let validity = v8::Local::new(scope, &x.validity);
    let _ = super::validity_state::replace(
        scope,
        validity,
        super::validity_state::ValidityRecord {
            value_missing: x.required && selected_index(scope, a.this()) < 0,
            custom_error: !x.custom_validity.is_empty(),
            ..Default::default()
        },
    );
    r.set(validity.into())
}
