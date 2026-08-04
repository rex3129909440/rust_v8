#[derive(Default)]
pub(crate) struct PrintState {
    jobs_started: u64,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(PrintState::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let function =
        crate::webidl::create_function(scope, "print", 0, v8::ConstructorBehavior::Throw, print)?;
    let global = scope.get_current_context().global(scope);
    let key = crate::webidl::string(scope, "print")?;
    if global.define_own_property(
        scope,
        key.into(),
        function.into(),
        v8::PropertyAttribute::NONE,
    ) == Some(true)
    {
        Ok(())
    } else {
        Err("cannot define window.print".to_owned())
    }
}

fn print(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if let Some(state) = scope.get_slot_mut::<PrintState>() {
        state.jobs_started = state.jobs_started.saturating_add(1);
    }
    let global = scope.get_current_context().global(scope);
    let event = super::event_target::create_event(scope, "beforeprint");
    super::event_target::dispatch(scope, global, event);
}
