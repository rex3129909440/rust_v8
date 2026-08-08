use super::event::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "initEvent", 1, init_event)
}

fn init_event(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let event_type = crate::webidl::value_to_string(scope, arguments.get(0));
    let bubbles = arguments.get(1).boolean_value(scope);
    let cancelable = arguments.get(2).boolean_value(scope);
    if record(scope, arguments.this()).is_some_and(|record| record.dispatching) {
        return;
    }
    update(scope, arguments.this(), |record| {
        record.event_type = event_type;
        record.bubbles = bubbles;
        record.cancelable = cancelable;
        record.composed = false;
        record.default_prevented = false;
        record.target = None;
        record.current_target = None;
        record.event_phase = NONE;
        record.cancel_bubble = false;
        record.immediate_stopped = false;
        record.initialized = true;
        record.path.clear();
    });
}
