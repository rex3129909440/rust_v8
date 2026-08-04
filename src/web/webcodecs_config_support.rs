pub(crate) fn dictionary<'s>(
    scope: &v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Value>,
) -> v8::Local<'s, v8::Object> {
    if value.is_null_or_undefined() {
        v8::Object::new(scope)
    } else {
        value
            .to_object(scope)
            .unwrap_or_else(|| v8::Object::new(scope))
    }
}

pub(crate) fn member<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'s, v8::Object>,
    name: &str,
) -> Option<v8::Local<'s, v8::Value>> {
    let key = v8::String::new(scope, name)?;
    object
        .get(scope, key.into())
        .filter(|value| !value.is_undefined())
}

pub(crate) fn string_member(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<String> {
    member(scope, object, name).map(|value| crate::webidl::value_to_string(scope, value))
}

pub(crate) fn number_member(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<f64> {
    member(scope, object, name)?.number_value(scope)
}

pub(crate) fn boolean_member(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    fallback: bool,
) -> bool {
    member(scope, object, name).map_or(fallback, |value| value.boolean_value(scope))
}

pub(crate) fn define_string(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: &str,
) {
    let Some(key) = v8::String::new(scope, name) else {
        return;
    };
    let Some(value) = v8::String::new(scope, value) else {
        return;
    };
    let _ = object.create_data_property(scope, key.into(), value.into());
}

pub(crate) fn define_number(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: f64,
) {
    let Some(key) = v8::String::new(scope, name) else {
        return;
    };
    let _ = object.create_data_property(scope, key.into(), v8::Number::new(scope, value).into());
}

pub(crate) fn define_boolean(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: bool,
) {
    let Some(key) = v8::String::new(scope, name) else {
        return;
    };
    let _ = object.create_data_property(scope, key.into(), v8::Boolean::new(scope, value).into());
}

pub(crate) fn reject_type_error(
    scope: &mut v8::PinScope<'_, '_>,
    message: &str,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(message) = v8::String::new(scope, message) else {
        return;
    };
    let exception = v8::Exception::type_error(scope, message);
    if let Ok(promise) = super::writable_stream::rejected_promise(scope, exception) {
        result.set(promise.into());
    }
}

pub(crate) fn resolve_support(
    scope: &mut v8::PinScope<'_, '_>,
    config: v8::Local<'_, v8::Object>,
    supported: bool,
    mut result: v8::ReturnValue<'_>,
) {
    let output = v8::Object::new(scope);
    if let Some(key) = v8::String::new(scope, "config") {
        let _ = output.create_data_property(scope, key.into(), config.into());
    }
    if let Some(key) = v8::String::new(scope, "supported") {
        let _ = output.create_data_property(
            scope,
            key.into(),
            v8::Boolean::new(scope, supported).into(),
        );
    }
    if let Ok(promise) = super::writable_stream::resolved_promise(scope, output.into()) {
        result.set(promise.into());
    }
}

pub(crate) fn codec_supported(patterns: &[String], codec: &str) -> bool {
    crate::fingerprint_environment::media_type_matches(patterns, codec)
}
