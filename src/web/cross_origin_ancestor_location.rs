pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let object = v8::Object::new(scope);
    let getter = crate::webidl::create_function(
        scope,
        "get href",
        0,
        v8::ConstructorBehavior::Throw,
        get_href,
    )?;
    let setter = crate::webidl::create_function(
        scope,
        "set href",
        1,
        v8::ConstructorBehavior::Throw,
        set_href,
    )?;
    let mut href_descriptor =
        v8::PropertyDescriptor::new_from_get_set(getter.into(), setter.into());
    href_descriptor.set_enumerable(true);
    href_descriptor.set_configurable(true);
    let href = crate::webidl::string(scope, "href")?;
    if object.define_property(scope, href.into(), &href_descriptor) != Some(true) {
        return Err("cannot define ancestor cross-origin Location.href".to_owned());
    }
    let replace = crate::webidl::create_function(
        scope,
        "replace",
        1,
        v8::ConstructorBehavior::Throw,
        replace,
    )?;
    let replace_key = crate::webidl::string(scope, "replace")?;
    if object.define_own_property(
        scope,
        replace_key.into(),
        replace.into(),
        v8::PropertyAttribute::DONT_ENUM,
    ) != Some(true)
    {
        return Err("cannot define ancestor cross-origin Location.replace".to_owned());
    }
    Ok(object)
}

fn get_href(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    super::cross_origin_location::throw_security_error(scope, "href", "Location");
}

fn set_href(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    navigate(
        scope,
        crate::webidl::value_to_string(scope, arguments.get(0)),
    );
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
    navigate(
        scope,
        crate::webidl::value_to_string(scope, arguments.get(0)),
    );
}

pub(crate) fn navigate(scope: &mut v8::PinScope<'_, '_>, value: String) {
    crate::page_init::navigate(scope, &value);
    if let Some(location) = super::location_global::value(scope) {
        let Some(href) = v8::String::new(scope, "href") else {
            return;
        };
        let Some(value) = v8::String::new(scope, &value) else {
            return;
        };
        let _ = location.set(scope, href.into(), value.into());
    }
}
