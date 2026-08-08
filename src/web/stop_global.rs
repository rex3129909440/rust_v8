#[derive(Default)]
pub(crate) struct StopState {
    stop_requests: u64,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(StopState::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let function =
        crate::webidl::create_function(scope, "stop", 0, v8::ConstructorBehavior::Throw, stop)?;
    let global = scope.get_current_context().global(scope);
    let key = crate::webidl::string(scope, "stop")?;
    if global.define_own_property(
        scope,
        key.into(),
        function.into(),
        v8::PropertyAttribute::NONE,
    ) == Some(true)
    {
        Ok(())
    } else {
        Err("cannot define window.stop".to_owned())
    }
}

fn stop(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if let Some(state) = scope.get_slot_mut::<StopState>() {
        state.stop_requests = state.stop_requests.saturating_add(1);
    }
}
