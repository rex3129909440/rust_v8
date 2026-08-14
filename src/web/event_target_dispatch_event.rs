use super::event_target::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(
        scope,
        prototype,
        "dispatchEvent",
        1,
        dispatch_event_callback,
    )
}

fn dispatch_event_callback(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !is_event_target(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let Ok(event) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "dispatchEvent requires an Event");
        return;
    };
    // DOM dispatchEvent() always establishes a synthetic dispatch, even if
    // script retained an Event object originally produced by the host.
    super::event::set_trusted(scope, event, false);
    let dispatched = dispatch(scope, arguments.this(), event);
    result.set(v8::Boolean::new(scope, dispatched).into());
}
