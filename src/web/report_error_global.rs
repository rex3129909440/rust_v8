use std::collections::VecDeque;

#[derive(Default)]
pub(crate) struct ReportErrorState {
    pending: VecDeque<v8::Global<v8::Object>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(ReportErrorState::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let function = crate::webidl::create_function(
        scope,
        "reportError",
        1,
        v8::ConstructorBehavior::Throw,
        execute,
    )?;
    let global = scope.get_current_context().global(scope);
    let key = crate::webidl::string(scope, "reportError")?;
    if global.define_own_property(
        scope,
        key.into(),
        function.into(),
        v8::PropertyAttribute::NONE,
    ) == Some(true)
    {
        Ok(())
    } else {
        Err("cannot define window.reportError".to_owned())
    }
}

pub(crate) fn execute(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'reportError' on 'Window': 1 argument required, but only 0 present.",
        );
        return;
    }
    let error = arguments.get(0);
    let description = error
        .to_string(scope)
        .map(|value| value.to_rust_string_lossy(scope))
        .unwrap_or_else(|| "undefined".to_owned());
    let message = format!("Uncaught {description}");
    let Ok(event) = super::error_event::create(scope, "error", message, error) else {
        return;
    };
    let stored_event = v8::Global::new(scope, event);
    if let Some(state) = scope.get_slot_mut::<ReportErrorState>() {
        state.pending.push_back(stored_event);
    }
    if let Some(task) = v8::Function::new(scope, dispatch_next) {
        scope.enqueue_microtask(task);
    }
}

fn dispatch_next(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let event = scope
        .get_slot_mut::<ReportErrorState>()
        .and_then(|state| state.pending.pop_front());
    if let Some(event) = event {
        let event = v8::Local::new(scope, &event);
        let global = scope.get_current_context().global(scope);
        super::event_target::dispatch(scope, global, event);
    }
}
