pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "removeAttributeNode", 1, call)
}

fn call(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if super::element::record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let Ok(attribute) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "The provided value is not an Attr");
        return;
    };
    let Some(attribute_record) = super::attr::record(scope, attribute) else {
        crate::webidl::throw_type_error(scope, "The provided value is not an Attr");
        return;
    };
    let owned = attribute_record
        .owner_element
        .as_ref()
        .is_some_and(|owner| v8::Local::new(scope, owner).strict_equals(arguments.this().into()));
    if !owned {
        super::node::throw_dom_exception(
            scope,
            "NotFoundError",
            "The attribute is not owned by this Element",
        );
        return;
    }
    if let Some(namespace) = attribute_record.namespace_uri {
        let namespace = v8::String::new(scope, &namespace)
            .map(Into::into)
            .unwrap_or_else(|| v8::undefined(scope).into());
        let local_name = v8::String::new(scope, &attribute_record.local_name)
            .map(Into::into)
            .unwrap_or_else(|| v8::undefined(scope).into());
        super::element::call_attribute_map_method(
            scope,
            arguments.this(),
            "removeNamedItemNS",
            &[namespace, local_name],
            result,
        );
    } else {
        let name = v8::String::new(scope, &attribute_record.name)
            .map(Into::into)
            .unwrap_or_else(|| v8::undefined(scope).into());
        super::element::call_attribute_map_method(
            scope,
            arguments.this(),
            "removeNamedItem",
            &[name],
            result,
        );
    }
}
