#[derive(Default)]
pub(crate) struct ConfirmState {
    last_message: String,
    next_response: bool,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(ConfirmState::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let function = crate::webidl::create_function(
        scope,
        "confirm",
        0,
        v8::ConstructorBehavior::Throw,
        confirm,
    )?;
    let global = scope.get_current_context().global(scope);
    let key = crate::webidl::string(scope, "confirm")?;
    if global.define_own_property(
        scope,
        key.into(),
        function.into(),
        v8::PropertyAttribute::NONE,
    ) == Some(true)
    {
        Ok(())
    } else {
        Err("cannot define window.confirm".to_owned())
    }
}

fn confirm(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let message = if arguments.length() == 0 {
        String::new()
    } else {
        crate::webidl::value_to_string(scope, arguments.get(0))
    };
    let response = if let Some(state) = scope.get_slot_mut::<ConfirmState>() {
        state.last_message = message;
        let response = state.next_response;
        state.next_response = false;
        response
    } else {
        false
    };
    result.set(v8::Boolean::new(scope, response).into());
}
