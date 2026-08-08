use super::html_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "attachInternals", 0, attach_internals)
}

pub(crate) fn attach_internals(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let existing = record(scope, arguments.this()).map(|record| record.internals);
    let Some(existing) = existing else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(existing) = existing {
        result.set(v8::Local::new(scope, &existing).into());
        return;
    }
    match super::element_internals::create(scope, None, None) {
        Ok(internals) => {
            let stored = v8::Global::new(scope, internals);
            if let Some(record) = scope.get_slot_mut::<HtmlElementStore>().and_then(|store| {
                store
                    .records
                    .get_mut(&arguments.this().get_identity_hash().get())
            }) {
                record.internals = Some(stored);
            }
            result.set(internals.into());
        }
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
