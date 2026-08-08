use super::pointer_event::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(
        scope,
        prototype,
        "getCoalescedEvents",
        0,
        get_coalesced_events,
    )
}

fn get_coalesced_events(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let events = v8::Array::new(scope, 1);
    let _ = events.set_index(scope, 0, arguments.this().into());
    result.set(events.into());
}
