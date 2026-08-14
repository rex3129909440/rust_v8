pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    event_target: v8::Local<'s, v8::Function>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let event_target_prototype = crate::webidl::prototype(scope, event_target)?;
    let template = v8::ObjectTemplate::new(scope);
    template.set_named_property_handler(
        v8::NamedPropertyHandlerConfiguration::new()
            .getter(named_getter)
            .query(named_query)
            .enumerator(named_enumerator)
            .descriptor(named_descriptor),
    );
    let properties = template
        .new_instance(scope)
        .ok_or_else(|| "cannot create WindowProperties exotic object".to_owned())?;
    if crate::webidl::set_platform_prototype(scope, properties, event_target_prototype.into())
        != Some(true)
    {
        return Err("cannot set WindowProperties inheritance".to_owned());
    }
    let tag_key = v8::Symbol::get_to_string_tag(scope);
    let tag = crate::webidl::string(scope, "WindowProperties")?;
    if properties.define_own_property(
        scope,
        tag_key.into(),
        tag.into(),
        v8::PropertyAttribute::READ_ONLY | v8::PropertyAttribute::DONT_ENUM,
    ) != Some(true)
    {
        return Err("cannot define WindowProperties toStringTag".to_owned());
    }
    super::root_window_proxy::register_window_properties(scope, properties)?;
    Ok(properties)
}

fn named_getter(
    scope: &mut v8::PinScope<'_, '_>,
    key: v8::Local<'_, v8::Name>,
    arguments: v8::PropertyCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_, v8::Value>,
) -> v8::Intercepted {
    crate::trace::record_named_native_intercept(scope, &arguments, "get", key, None);
    if key.is_symbol() {
        return v8::Intercepted::kNo;
    }
    let name = crate::webidl::value_to_string(scope, key.into());
    let Some(value) =
        super::root_window_proxy::document_named_value(scope, arguments.holder(), &name)
    else {
        return v8::Intercepted::kNo;
    };
    result.set(v8::Local::new(scope, &value));
    v8::Intercepted::kYes
}

fn named_query(
    scope: &mut v8::PinScope<'_, '_>,
    key: v8::Local<'_, v8::Name>,
    arguments: v8::PropertyCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_, v8::Integer>,
) -> v8::Intercepted {
    crate::trace::record_named_native_intercept(scope, &arguments, "has", key, None);
    if key.is_symbol() {
        return v8::Intercepted::kNo;
    }
    let name = crate::webidl::value_to_string(scope, key.into());
    if super::root_window_proxy::document_named_value(scope, arguments.holder(), &name).is_none() {
        return v8::Intercepted::kNo;
    }
    result.set_int32(0);
    v8::Intercepted::kYes
}

fn named_enumerator(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::PropertyCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_, v8::Array>,
) {
    crate::trace::record_native_enumeration(scope, &arguments);
    result.set(v8::Array::new(scope, 0));
}

fn named_descriptor(
    scope: &mut v8::PinScope<'_, '_>,
    key: v8::Local<'_, v8::Name>,
    arguments: v8::PropertyCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_, v8::Value>,
) -> v8::Intercepted {
    crate::trace::record_named_native_intercept(
        scope,
        &arguments,
        "getOwnPropertyDescriptor",
        key,
        None,
    );
    if key.is_symbol() {
        return v8::Intercepted::kNo;
    }
    let name = crate::webidl::value_to_string(scope, key.into());
    let Some(value) =
        super::root_window_proxy::document_named_value(scope, arguments.holder(), &name)
    else {
        return v8::Intercepted::kNo;
    };
    let value = v8::Local::new(scope, &value);
    let descriptor =
        super::cross_origin_window_descriptors::data_descriptor(scope, value, true, false, true);
    result.set(descriptor.into());
    v8::Intercepted::kYes
}
