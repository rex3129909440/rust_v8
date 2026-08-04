pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "createElementNS", 2, create_element_ns)
}

fn create_element_ns(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !super::document::is_document(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let namespace = if arguments.get(0).is_null() {
        None
    } else {
        let namespace = crate::webidl::value_to_string(scope, arguments.get(0));
        (!namespace.is_empty()).then_some(namespace)
    };
    let qualified_name = crate::webidl::value_to_string(scope, arguments.get(1));
    if let Err((name, message)) =
        super::document::validate_qualified_name(namespace.as_deref(), &qualified_name, false)
    {
        super::node::throw_dom_exception(scope, name, message);
        return;
    }
    let local_name = qualified_name
        .rsplit_once(':')
        .map(|(_, local_name)| local_name)
        .unwrap_or(&qualified_name);
    let created = match namespace.as_deref() {
        Some("http://www.w3.org/2000/svg") => {
            super::document::create_svg_element(scope, local_name)
        }
        Some("http://www.w3.org/1998/Math/MathML") => {
            super::math_ml_element::create(scope, local_name.to_owned())
        }
        _ => super::element::create(scope, qualified_name.clone(), namespace.clone()),
    };
    match created {
        Ok(element) => {
            super::element::set_qualified_name(scope, element, qualified_name);
            super::node::set_owner_document(scope, element, arguments.this());
            result.set(element.into());
        }
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
