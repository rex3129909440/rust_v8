use super::navigate_event::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "intercept", 0, intercept)
}

fn intercept(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let id = arguments.this().get_identity_hash().get();
    let Some(snapshot) = scope
        .get_slot::<NavigateEventStore>()
        .and_then(|store| store.records.get(&id))
        .cloned()
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if !snapshot.trusted_navigation {
        throw_dom_exception(
            scope,
            "SecurityError",
            "intercept() may only be called on a trusted navigate event",
        );
        return;
    }
    if !snapshot.can_intercept {
        throw_dom_exception(
            scope,
            "SecurityError",
            "This navigation cannot be intercepted",
        );
        return;
    }
    let handler = v8::Local::<v8::Object>::try_from(arguments.get(0))
        .ok()
        .and_then(|options| value_property(scope, options, "handler"))
        .and_then(|value| v8::Local::<v8::Function>::try_from(value).ok())
        .map(|function| v8::Global::new(scope, function));
    if let Some(record) = scope
        .get_slot_mut::<NavigateEventStore>()
        .and_then(|store| store.records.get_mut(&id))
    {
        record.intercepted = true;
        if let Some(handler) = handler {
            record.handlers.push(handler);
        }
    }
}
