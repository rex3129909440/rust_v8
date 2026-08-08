pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "attachShadow", 1, attach_shadow)
}

fn attach_shadow(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(element) = super::element::record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let Ok(options) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'attachShadow': parameter 1 is not an object",
        );
        return;
    };
    let Some(mode_value) = member(scope, options, "mode") else {
        crate::webidl::throw_type_error(scope, "The required member 'mode' is undefined");
        return;
    };
    let mode = crate::webidl::value_to_string(scope, mode_value);
    if !matches!(mode.as_str(), "open" | "closed") {
        crate::webidl::throw_type_error(scope, "'mode' must be either 'open' or 'closed'");
        return;
    }
    if element.shadow_root.is_some() {
        super::node::throw_dom_exception(
            scope,
            "NotSupportedError",
            "This element already hosts a shadow root",
        );
        return;
    }
    if !valid_shadow_host(&element) {
        super::node::throw_dom_exception(
            scope,
            "NotSupportedError",
            "This element does not support attachShadow",
        );
        return;
    }
    let slot_assignment = member(scope, options, "slotAssignment")
        .map(|value| crate::webidl::value_to_string(scope, value))
        .unwrap_or_else(|| "named".to_owned());
    if !matches!(slot_assignment.as_str(), "named" | "manual") {
        crate::webidl::throw_type_error(
            scope,
            &format!(
                "'slotAssignment' must be either 'named' or 'manual', received '{slot_assignment}'"
            ),
        );
        return;
    }
    let delegates_focus = bool_member(scope, options, "delegatesFocus");
    let serializable = bool_member(scope, options, "serializable");
    let clonable = bool_member(scope, options, "clonable");
    let registry = member(scope, options, "customElementRegistry")
        .filter(|value| !value.is_null_or_undefined())
        .and_then(|value| v8::Local::<v8::Object>::try_from(value).ok());
    match super::shadow_root::create(
        scope,
        arguments.this(),
        mode,
        delegates_focus,
        slot_assignment,
        serializable,
        clonable,
        registry,
    ) {
        Ok(shadow) => {
            super::element::set_shadow_root(scope, arguments.this(), shadow);
            result.set(shadow.into());
        }
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn member<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<v8::Local<'s, v8::Value>> {
    object
        .get(scope, v8::String::new(scope, name)?.into())
        .filter(|value| !value.is_undefined())
}

fn bool_member(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> bool {
    member(scope, object, name).is_some_and(|value| value.boolean_value(scope))
}

fn valid_shadow_host(record: &super::element::ElementRecord) -> bool {
    if record.namespace_uri.as_deref() != Some("http://www.w3.org/1999/xhtml") {
        return false;
    }
    let local_name = record
        .tag_name
        .rsplit(':')
        .next()
        .unwrap_or(&record.tag_name)
        .to_ascii_lowercase();
    local_name.contains('-')
        || matches!(
            local_name.as_str(),
            "article"
                | "aside"
                | "blockquote"
                | "body"
                | "div"
                | "footer"
                | "h1"
                | "h2"
                | "h3"
                | "h4"
                | "h5"
                | "h6"
                | "header"
                | "main"
                | "nav"
                | "p"
                | "section"
                | "span"
        )
}
