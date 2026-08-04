#[derive(Default)]
pub(crate) struct PromptState {
    last_message: String,
    last_default: String,
    next_response: Option<String>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(PromptState::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let function =
        crate::webidl::create_function(scope, "prompt", 0, v8::ConstructorBehavior::Throw, prompt)?;
    let global = scope.get_current_context().global(scope);
    let key = crate::webidl::string(scope, "prompt")?;
    if global.define_own_property(
        scope,
        key.into(),
        function.into(),
        v8::PropertyAttribute::NONE,
    ) == Some(true)
    {
        Ok(())
    } else {
        Err("cannot define window.prompt".to_owned())
    }
}

fn prompt(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let message = if arguments.get(0).is_undefined() {
        String::new()
    } else {
        crate::webidl::value_to_string(scope, arguments.get(0))
    };
    let default_value = if arguments.get(1).is_undefined() {
        String::new()
    } else {
        crate::webidl::value_to_string(scope, arguments.get(1))
    };
    let response = if let Some(state) = scope.get_slot_mut::<PromptState>() {
        state.last_message = message;
        state.last_default = default_value;
        state.next_response.take()
    } else {
        None
    };
    if let Some(response) = response {
        if let Some(response) = v8::String::new(scope, &response) {
            result.set(response.into());
        }
    } else {
        result.set(v8::null(scope).into());
    }
}
