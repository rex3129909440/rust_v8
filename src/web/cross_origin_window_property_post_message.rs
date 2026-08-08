pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    iframe_id: i32,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let data = v8::Integer::new(scope, iframe_id);
    crate::webidl::create_function_with_data(
        scope,
        "postMessage",
        1,
        v8::ConstructorBehavior::Throw,
        post_message,
        data.into(),
    )
}

pub(crate) fn create_ancestor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    crate::webidl::create_function(
        scope,
        "postMessage",
        1,
        v8::ConstructorBehavior::Throw,
        super::post_message_global::post_message,
    )
}

fn post_message(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let data = crate::trace::native_callback_data(scope, &arguments);
    let iframe_id = data.int32_value(scope).unwrap_or_default();
    let Some((target, parent)) =
        super::html_i_frame_element::cross_origin_message_windows(scope, iframe_id)
    else {
        crate::webidl::throw_type_error(scope, "Cross-origin WindowProxy is detached");
        return;
    };
    let Some(key) = v8::String::new(scope, "postMessage") else {
        return;
    };
    let Some(function) = parent
        .get(scope, key.into())
        .and_then(|value| v8::Local::<v8::Function>::try_from(value).ok())
    else {
        crate::webidl::throw_type_error(scope, "Window.postMessage is unavailable");
        return;
    };
    let value = match arguments.length() {
        0 => function.call(scope, target.into(), &[]),
        1 => function.call(scope, target.into(), &[arguments.get(0)]),
        2 => function.call(scope, target.into(), &[arguments.get(0), arguments.get(1)]),
        _ => function.call(
            scope,
            target.into(),
            &[arguments.get(0), arguments.get(1), arguments.get(2)],
        ),
    };
    if let Some(value) = value {
        result.set(value);
    }
}
