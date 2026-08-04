#[derive(Default)]
pub(crate) struct CaptureEventsState {
    event_mask: u32,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CaptureEventsState::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let function = crate::webidl::create_function(
        scope,
        "captureEvents",
        0,
        v8::ConstructorBehavior::Throw,
        capture_events,
    )?;
    let global = scope.get_current_context().global(scope);
    let key = crate::webidl::string(scope, "captureEvents")?;
    if global.define_own_property(
        scope,
        key.into(),
        function.into(),
        v8::PropertyAttribute::NONE,
    ) == Some(true)
    {
        Ok(())
    } else {
        Err("cannot define window.captureEvents".to_owned())
    }
}

fn capture_events(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let event_mask = arguments.get(0).uint32_value(scope).unwrap_or(0);
    if let Some(state) = scope.get_slot_mut::<CaptureEventsState>() {
        state.event_mask = event_mask;
    }
}

pub(crate) fn release(scope: &mut v8::PinScope<'_, '_>, event_mask: u32) {
    if let Some(state) = scope.get_slot_mut::<CaptureEventsState>() {
        state.event_mask &= !event_mask;
    }
}
