#[derive(Default)]
pub(crate) struct MoveToState {
    last_x: f64,
    last_y: f64,
    attempts: u64,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(MoveToState::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let function = crate::webidl::create_function(
        scope,
        "moveTo",
        2,
        v8::ConstructorBehavior::Throw,
        move_to,
    )?;
    let global = scope.get_current_context().global(scope);
    let key = crate::webidl::string(scope, "moveTo")?;
    if global.define_own_property(
        scope,
        key.into(),
        function.into(),
        v8::PropertyAttribute::NONE,
    ) == Some(true)
    {
        Ok(())
    } else {
        Err("cannot define window.moveTo".to_owned())
    }
}

fn move_to(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let x = arguments.get(0).number_value(scope).unwrap_or(0.0);
    let y = arguments.get(1).number_value(scope).unwrap_or(0.0);
    if let Some(state) = scope.get_slot_mut::<MoveToState>() {
        state.last_x = x;
        state.last_y = y;
        state.attempts = state.attempts.saturating_add(1);
    }
}
