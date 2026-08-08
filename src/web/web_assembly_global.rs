pub(crate) fn install(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> Result<(), String> {
    let object = v8::Local::<v8::Object>::try_from(value)
        .map_err(|_| "WebAssembly namespace is not an object".to_owned())?;
    let suspending = take_property(scope, object, "Suspending")?;
    let promising = take_property(scope, object, "promising")?;
    let suspend_error = take_property(scope, object, "SuspendError")?;
    let tag_key = v8::Symbol::get_to_string_tag(scope);
    let tag = object
        .get(scope, tag_key.into())
        .ok_or_else(|| "WebAssembly namespace has no toStringTag".to_owned())?;
    if object.delete(scope, tag_key.into()) != Some(true) {
        return Err("cannot reorder WebAssembly toStringTag".to_owned());
    }
    crate::webidl::define_method(scope, object, "compileStreaming", 1, compile_streaming)?;
    crate::webidl::define_method(
        scope,
        object,
        "instantiateStreaming",
        1,
        instantiate_streaming,
    )?;
    restore_property(
        scope,
        object,
        "Suspending",
        suspending,
        v8::PropertyAttribute::DONT_ENUM,
    )?;
    restore_property(
        scope,
        object,
        "promising",
        promising,
        v8::PropertyAttribute::NONE,
    )?;
    restore_property(
        scope,
        object,
        "SuspendError",
        suspend_error,
        v8::PropertyAttribute::DONT_ENUM,
    )?;
    if object.define_own_property(
        scope,
        tag_key.into(),
        tag,
        v8::PropertyAttribute::READ_ONLY | v8::PropertyAttribute::DONT_ENUM,
    ) != Some(true)
    {
        return Err("cannot restore WebAssembly toStringTag".to_owned());
    }
    let global = scope.get_current_context().global(scope);
    let key = crate::webidl::string(scope, "WebAssembly")?;
    if global.define_own_property(scope, key.into(), value, v8::PropertyAttribute::DONT_ENUM)
        == Some(true)
    {
        Ok(())
    } else {
        Err("cannot define window.WebAssembly".to_owned())
    }
}

fn take_property<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'s, v8::Object>,
    name: &str,
) -> Result<v8::Local<'s, v8::Value>, String> {
    let key = crate::webidl::string(scope, name)?;
    let value = object
        .get(scope, key.into())
        .ok_or_else(|| format!("WebAssembly.{name} is unavailable"))?;
    if object.delete(scope, key.into()) == Some(true) {
        Ok(value)
    } else {
        Err(format!("cannot reorder WebAssembly.{name}"))
    }
}

fn restore_property(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: v8::Local<'_, v8::Value>,
    attributes: v8::PropertyAttribute,
) -> Result<(), String> {
    let key = crate::webidl::string(scope, name)?;
    if object.define_own_property(scope, key.into(), value, attributes) == Some(true) {
        Ok(())
    } else {
        Err(format!("cannot restore WebAssembly.{name}"))
    }
}

fn compile_streaming(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    forward(scope, arguments, result, "compile");
}

fn instantiate_streaming(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    forward(scope, arguments, result, "instantiate");
}

fn forward(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    method_name: &str,
) {
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            &format!("WebAssembly.{method_name}Streaming requires one argument"),
        );
        return;
    }
    let global = scope.get_current_context().global(scope);
    let Some(promise_key) = v8::String::new(scope, "Promise") else {
        return;
    };
    let Some(promise_constructor) = global.get(scope, promise_key.into()) else {
        return;
    };
    let Ok(promise_constructor) = v8::Local::<v8::Object>::try_from(promise_constructor) else {
        return;
    };
    let Some(resolve_key) = v8::String::new(scope, "resolve") else {
        return;
    };
    let Some(resolve) = promise_constructor.get(scope, resolve_key.into()) else {
        return;
    };
    let Ok(resolve) = v8::Local::<v8::Function>::try_from(resolve) else {
        return;
    };
    let Some(source_promise) = resolve.call(scope, promise_constructor.into(), &[arguments.get(0)])
    else {
        return;
    };
    let Ok(source_promise) = v8::Local::<v8::Object>::try_from(source_promise) else {
        return;
    };

    let extract = v8::Function::builder(extract_stream_bytes)
        .length(1)
        .constructor_behavior(v8::ConstructorBehavior::Throw)
        .build(scope);
    let Some(extract) = extract else {
        return;
    };
    let Some(bytes_promise) = call_then(scope, source_promise, extract) else {
        return;
    };

    let callback_data = v8::Array::new(scope, 3);
    let Some(method_name) = v8::String::new(scope, method_name) else {
        return;
    };
    let _ = callback_data.set_index(scope, 0, method_name.into());
    let second = if arguments.length() > 1 {
        arguments.get(1)
    } else {
        v8::undefined(scope).into()
    };
    let third = if arguments.length() > 2 {
        arguments.get(2)
    } else {
        v8::undefined(scope).into()
    };
    let _ = callback_data.set_index(scope, 1, second);
    let _ = callback_data.set_index(scope, 2, third);
    let consume = v8::Function::builder(consume_stream_bytes)
        .data(callback_data.into())
        .length(1)
        .constructor_behavior(v8::ConstructorBehavior::Throw)
        .build(scope);
    let Some(consume) = consume else {
        return;
    };
    if let Some(promise) = call_then(scope, bytes_promise, consume) {
        result.set(promise.into());
    }
}

fn call_then<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    promise: v8::Local<'s, v8::Object>,
    callback: v8::Local<'s, v8::Function>,
) -> Option<v8::Local<'s, v8::Object>> {
    let key = v8::String::new(scope, "then")?;
    let then = promise.get(scope, key.into())?;
    let then = v8::Local::<v8::Function>::try_from(then).ok()?;
    let value = then.call(scope, promise.into(), &[callback.into()])?;
    v8::Local::<v8::Object>::try_from(value).ok()
}

fn extract_stream_bytes(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let source = arguments.get(0);
    let Ok(response) = v8::Local::<v8::Object>::try_from(source) else {
        crate::webidl::throw_type_error(scope, "WebAssembly streaming source must be a Response");
        return;
    };
    if let Err(message) = super::response::validate_wasm_streaming_source(scope, response) {
        crate::webidl::throw_type_error(scope, message);
        return;
    };
    let Some(key) = v8::String::new(scope, "arrayBuffer") else {
        return;
    };
    let Some(method) = response.get(scope, key.into()) else {
        result.set(source);
        return;
    };
    let Ok(method) = v8::Local::<v8::Function>::try_from(method) else {
        result.set(source);
        return;
    };
    if let Some(value) = method.call(scope, response.into(), &[]) {
        result.set(value);
    }
}

fn consume_stream_bytes(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Ok(data) = v8::Local::<v8::Array>::try_from(arguments.data()) else {
        return;
    };
    let Some(method_name) = data
        .get_index(scope, 0)
        .and_then(|value| value.to_string(scope))
    else {
        return;
    };
    let second = data
        .get_index(scope, 1)
        .unwrap_or_else(|| v8::undefined(scope).into());
    let third = data
        .get_index(scope, 2)
        .unwrap_or_else(|| v8::undefined(scope).into());
    let global = scope.get_current_context().global(scope);
    let Some(namespace_key) = v8::String::new(scope, "WebAssembly") else {
        return;
    };
    let Some(namespace) = global.get(scope, namespace_key.into()) else {
        return;
    };
    let Ok(namespace) = v8::Local::<v8::Object>::try_from(namespace) else {
        return;
    };
    let Some(module_key) = v8::String::new(scope, "Module") else {
        return;
    };
    let Some(module_constructor) = namespace.get(scope, module_key.into()) else {
        return;
    };
    let Ok(module_constructor) = v8::Local::<v8::Function>::try_from(module_constructor) else {
        return;
    };
    let module = if method_name.to_rust_string_lossy(scope) == "compile" {
        if second.is_undefined() {
            module_constructor.new_instance(scope, &[arguments.get(0)])
        } else {
            module_constructor.new_instance(scope, &[arguments.get(0), second])
        }
    } else if third.is_undefined() {
        module_constructor.new_instance(scope, &[arguments.get(0)])
    } else {
        module_constructor.new_instance(scope, &[arguments.get(0), third])
    };
    let Some(module) = module else {
        return;
    };
    if method_name.to_rust_string_lossy(scope) == "compile" {
        result.set(module.into());
        return;
    }

    let Some(instance_key) = v8::String::new(scope, "Instance") else {
        return;
    };
    let Some(instance_constructor) = namespace.get(scope, instance_key.into()) else {
        return;
    };
    let Ok(instance_constructor) = v8::Local::<v8::Function>::try_from(instance_constructor) else {
        return;
    };
    let instance = if second.is_undefined() {
        instance_constructor.new_instance(scope, &[module.into()])
    } else {
        instance_constructor.new_instance(scope, &[module.into(), second])
    };
    let Some(instance) = instance else {
        return;
    };
    let output = v8::Object::new(scope);
    let Some(module_key) = v8::String::new(scope, "module") else {
        return;
    };
    let Some(instance_key) = v8::String::new(scope, "instance") else {
        return;
    };
    if output.set(scope, module_key.into(), module.into()) == Some(true)
        && output.set(scope, instance_key.into(), instance.into()) == Some(true)
    {
        result.set(output.into());
    }
}
