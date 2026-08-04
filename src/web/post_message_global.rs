use std::collections::VecDeque;

#[derive(Clone)]
struct PendingWindowMessage {
    context: v8::Global<v8::Context>,
    target: v8::Global<v8::Object>,
    data: v8::Global<v8::Value>,
    origin: String,
    source: v8::Global<v8::Object>,
    ports: Vec<v8::Global<v8::Object>>,
}

#[derive(Default)]
pub(crate) struct PostMessageState {
    pending: VecDeque<PendingWindowMessage>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(PostMessageState::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let function = crate::webidl::create_function(
        scope,
        "postMessage",
        1,
        v8::ConstructorBehavior::Throw,
        post_message,
    )?;
    let global = scope.get_current_context().global(scope);
    let key = crate::webidl::string(scope, "postMessage")?;
    if global.define_own_property(
        scope,
        key.into(),
        function.into(),
        v8::PropertyAttribute::NONE,
    ) == Some(true)
    {
        Ok(())
    } else {
        Err("cannot define window.postMessage".to_owned())
    }
}

pub(crate) fn post_message(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'postMessage' on 'Window': 1 argument required, but only 0 present.",
        );
        return;
    }
    let target = arguments.this();
    let source_context = scope.get_entered_or_microtask_context();
    let source = source_context.global(scope);
    let target_context = if target.strict_equals(source.into()) {
        v8::Global::new(scope, source_context)
    } else {
        let Some(context) = super::html_i_frame_element::context_for_window(scope, target) else {
            crate::webidl::throw_type_error(scope, "Illegal invocation");
            return;
        };
        context
    };
    let source_origin = super::html_i_frame_element::origin_for_window(scope, source);
    let target_origin = super::html_i_frame_element::origin_for_window(scope, target);

    let (requested_origin, transfer) = match post_message_options(scope, &arguments) {
        Ok(options) => options,
        Err(message) => {
            super::structured_clone::throw_data_clone_error(scope, &message);
            return;
        }
    };
    let allowed = if requested_origin == "*" {
        true
    } else if requested_origin == "/" {
        source_origin == target_origin
    } else {
        let Ok(url) = url::Url::parse(&requested_origin) else {
            throw_syntax_error(scope, "Invalid target origin.");
            return;
        };
        if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
            throw_syntax_error(scope, "Invalid target origin.");
            return;
        }
        url.origin().ascii_serialization() == target_origin
    };
    if !allowed {
        return;
    }

    let target_context_local = v8::Local::new(scope, &target_context);
    let cloned = match super::structured_clone::clone_into(
        scope,
        target_context_local,
        arguments.get(0),
        transfer,
    ) {
        Ok(cloned) => cloned,
        Err(message) => {
            super::structured_clone::throw_data_clone_error(scope, &message);
            return;
        }
    };
    let pending = PendingWindowMessage {
        context: target_context,
        target: v8::Global::new(scope, target),
        data: cloned.value,
        origin: source_origin,
        source: v8::Global::new(scope, source),
        ports: cloned.ports,
    };
    if let Some(state) = scope.get_slot_mut::<PostMessageState>() {
        state.pending.push_back(pending);
    }
    if let Some(task) = v8::Function::new(scope, dispatch_next) {
        scope.enqueue_microtask(task);
    }
}

fn post_message_options(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: &v8::FunctionCallbackArguments<'_>,
) -> Result<(String, super::structured_clone::TransferList), String> {
    let second = arguments.get(1);
    if second.is_object() && !second.is_string_object() && !second.is_array() {
        let options = v8::Local::<v8::Object>::try_from(second)
            .map_err(|_| "postMessage options are invalid.".to_owned())?;
        let target_origin_key = crate::webidl::string(scope, "targetOrigin")?;
        let target_origin = options
            .get(scope, target_origin_key.into())
            .filter(|value| !value.is_undefined())
            .map(|value| crate::webidl::value_to_string(scope, value))
            .unwrap_or_else(|| "/".to_owned());
        let transfer = super::structured_clone::transfer_from_options(scope, second)?;
        return Ok((target_origin, transfer));
    }
    let target_origin = if second.is_undefined() {
        "/".to_owned()
    } else {
        crate::webidl::value_to_string(scope, second)
    };
    let transfer = super::structured_clone::transfer_from_sequence(scope, arguments.get(2))?;
    Ok((target_origin, transfer))
}

fn dispatch_next(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let message = scope
        .get_slot_mut::<PostMessageState>()
        .and_then(|state| state.pending.pop_front());
    let Some(message) = message else {
        return;
    };
    let context = v8::Local::new(scope, &message.context);
    let target_scope = &mut v8::ContextScope::new(scope, context);
    let target = v8::Local::new(target_scope, &message.target);
    let data = v8::Local::new(target_scope, &message.data);
    let source = v8::Local::new(target_scope, &message.source);
    let ports = message
        .ports
        .iter()
        .map(|port| v8::Local::new(target_scope, port))
        .collect();
    let Ok(event) = super::message_event::create(
        target_scope,
        "message",
        data,
        &message.origin,
        Some(source.into()),
        ports,
    ) else {
        return;
    };
    super::event_target::dispatch(target_scope, target, event);
}

fn throw_syntax_error(scope: &mut v8::PinScope<'_, '_>, message: &str) {
    if let Ok(exception) =
        super::dom_exception::create(scope, message.to_owned(), "SyntaxError".to_owned())
    {
        scope.throw_exception(exception.into());
    }
}
