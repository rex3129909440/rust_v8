pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    iframe_id: i32,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let object = v8::Object::new(scope);
    let data = v8::Integer::new(scope, iframe_id);
    let getter = crate::webidl::create_function_with_data(
        scope,
        "get href",
        0,
        v8::ConstructorBehavior::Throw,
        get_href,
        data.into(),
    )?;
    let setter = crate::webidl::create_function_with_data(
        scope,
        "set href",
        1,
        v8::ConstructorBehavior::Throw,
        set_href,
        data.into(),
    )?;
    let mut href_descriptor =
        v8::PropertyDescriptor::new_from_get_set(getter.into(), setter.into());
    href_descriptor.set_enumerable(true);
    href_descriptor.set_configurable(true);
    let href = crate::webidl::string(scope, "href")?;
    if object.define_property(scope, href.into(), &href_descriptor) != Some(true) {
        return Err("cannot define cross-origin Location.href".to_owned());
    }

    for name in [
        "origin",
        "protocol",
        "host",
        "hostname",
        "port",
        "pathname",
        "search",
        "hash",
        "ancestorOrigins",
    ] {
        define_blocked_property(scope, object, name, true)?;
    }
    for name in ["assign", "reload", "toString", "valueOf"] {
        define_blocked_property(scope, object, name, false)?;
    }

    let replace = crate::webidl::create_function_with_data(
        scope,
        "replace",
        1,
        v8::ConstructorBehavior::Throw,
        replace,
        data.into(),
    )?;
    let replace_key = crate::webidl::string(scope, "replace")?;
    if object.define_own_property(
        scope,
        replace_key.into(),
        replace.into(),
        v8::PropertyAttribute::DONT_ENUM,
    ) != Some(true)
    {
        return Err("cannot define cross-origin Location.replace".to_owned());
    }
    Ok(object)
}

fn define_blocked_property(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    enumerable: bool,
) -> Result<(), String> {
    let data = crate::webidl::string(scope, name)?;
    let getter = crate::webidl::create_function_with_data(
        scope,
        &format!("get {name}"),
        0,
        v8::ConstructorBehavior::Throw,
        get_blocked_property,
        data.into(),
    )?;
    let data = crate::webidl::string(scope, name)?;
    let setter = crate::webidl::create_function_with_data(
        scope,
        &format!("set {name}"),
        1,
        v8::ConstructorBehavior::Throw,
        set_blocked_property,
        data.into(),
    )?;
    let mut descriptor = v8::PropertyDescriptor::new_from_get_set(getter.into(), setter.into());
    descriptor.set_enumerable(enumerable);
    descriptor.set_configurable(true);
    let key = crate::webidl::string(scope, name)?;
    if object.define_property(scope, key.into(), &descriptor) == Some(true) {
        Ok(())
    } else {
        Err(format!("cannot define cross-origin Location.{name}"))
    }
}

fn get_blocked_property(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let property = crate::webidl::value_to_string(
        scope,
        crate::trace::native_callback_data(scope, &arguments),
    );
    throw_security_error(scope, &property, "Location");
}

fn set_blocked_property(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let property = crate::webidl::value_to_string(
        scope,
        crate::trace::native_callback_data(scope, &arguments),
    );
    throw_security_error(scope, &property, "Location");
}

fn get_href(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    throw_security_error(scope, "href", "Location");
}

fn set_href(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let data = crate::trace::native_callback_data(scope, &arguments);
    navigate(scope, data, arguments.get(0));
}

fn replace(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'replace' on 'Location': 1 argument required",
        );
        return;
    }
    let data = crate::trace::native_callback_data(scope, &arguments);
    navigate(scope, data, arguments.get(0));
}

fn navigate(
    scope: &mut v8::PinScope<'_, '_>,
    data: v8::Local<'_, v8::Value>,
    value: v8::Local<'_, v8::Value>,
) {
    let Some(iframe_id) = data.int32_value(scope) else {
        crate::webidl::throw_type_error(scope, "Cross-origin Location is detached");
        return;
    };
    let value = crate::webidl::value_to_string(scope, value);
    if let Err(message) =
        super::html_i_frame_element::navigate_cross_origin_location(scope, iframe_id, value)
    {
        crate::webidl::throw_type_error(scope, &message);
    }
}

pub(crate) fn throw_security_error(
    scope: &mut v8::PinScope<'_, '_>,
    property: &str,
    interface: &str,
) {
    let window = scope.get_current_context().global(scope);
    let origin = super::html_i_frame_element::origin_for_window(scope, window);
    let message = format!(
        "Failed to read a named property '{property}' from '{interface}': Blocked a frame with origin \"{origin}\" from accessing a cross-origin frame."
    );
    match super::dom_exception::create(scope, message, "SecurityError".to_owned()) {
        Ok(exception) => {
            scope.throw_exception(exception.into());
        }
        Err(_) => {
            let message = crate::webidl::string(
                scope,
                &format!(
                    "Blocked a frame with origin \"{origin}\" from accessing a cross-origin frame"
                ),
            )
            .expect("short SecurityError message");
            scope.throw_exception(v8::Exception::error(scope, message));
        }
    }
}
