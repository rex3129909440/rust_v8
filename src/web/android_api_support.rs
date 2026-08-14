pub(crate) fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let _ = arguments;
    crate::webidl::throw_type_error(scope, "Illegal constructor");
}

pub(crate) fn require_brand(
    scope: &mut v8::PinScope<'_, '_>,
    valid: bool,
    interface: &str,
    operation: &str,
) -> bool {
    if valid {
        true
    } else {
        crate::webidl::throw_type_error(
            scope,
            &format!("Failed to execute '{operation}' on '{interface}': Illegal invocation"),
        );
        false
    }
}

pub(crate) fn property<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'s, v8::Object>,
    name: &str,
) -> Option<v8::Local<'s, v8::Value>> {
    let key = v8::String::new(scope, name)?;
    object.get(scope, key.into())
}

pub(crate) fn string_property(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> String {
    property(scope, object, name)
        .filter(|value| !value.is_undefined() && !value.is_null())
        .map(|value| crate::webidl::value_to_string(scope, value))
        .unwrap_or_default()
}

pub(crate) fn set_tag(
    scope: &v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
    name: &str,
) -> Result<(), String> {
    let tag = v8::Symbol::get_to_string_tag(scope);
    let value = crate::webidl::string(scope, name)?;
    if prototype.define_own_property(
        scope,
        tag.into(),
        value.into(),
        v8::PropertyAttribute::READ_ONLY | v8::PropertyAttribute::DONT_ENUM,
    ) == Some(true)
    {
        Ok(())
    } else {
        Err(format!("cannot define {name} @@toStringTag"))
    }
}

pub(crate) fn resolved_undefined<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Option<v8::Local<'s, v8::Promise>> {
    let value = v8::undefined(scope);
    super::writable_stream::resolved_promise(scope, value.into()).ok()
}

pub(crate) fn rejected_dom_exception<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    name: &str,
    message: &str,
) -> Option<v8::Local<'s, v8::Promise>> {
    let exception =
        super::dom_exception::create(scope, message.to_owned(), name.to_owned()).ok()?;
    super::writable_stream::rejected_promise(scope, exception.into()).ok()
}
