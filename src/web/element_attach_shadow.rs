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
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'attachShadow' on 'Element': 1 argument required, but only 0 present.",
        );
        return;
    }
    let Ok(options) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'attachShadow' on 'Element': The provided value is not of type 'ShadowRootInit'.",
        );
        return;
    };

    // Web IDL converts dictionary members in lexicographic order, not in the
    // order in which the implementation happens to consume them.
    let Some(clonable_value) = raw_member(scope, options, "clonable") else {
        return;
    };
    let Some(registry_value) = raw_member(scope, options, "customElementRegistry") else {
        return;
    };
    let Some(delegates_focus_value) = raw_member(scope, options, "delegatesFocus") else {
        return;
    };
    let Some(mode_value) = raw_member(scope, options, "mode") else {
        return;
    };
    if mode_value.is_undefined() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'attachShadow' on 'Element': Failed to read the 'mode' property from 'ShadowRootInit': Required member is undefined.",
        );
        return;
    }
    let Some(mode) = crate::webidl::dom_string_with_context(
        scope,
        mode_value,
        "Failed to execute 'attachShadow' on 'Element': Failed to read the 'mode' property from 'ShadowRootInit'",
    ) else {
        return;
    };
    let Some(serializable_value) = raw_member(scope, options, "serializable") else {
        return;
    };
    let Some(slot_assignment_value) = raw_member(scope, options, "slotAssignment") else {
        return;
    };
    if !matches!(mode.as_str(), "open" | "closed") {
        crate::webidl::throw_type_error(
            scope,
            &format!(
                "Failed to execute 'attachShadow' on 'Element': Failed to read the 'mode' property from 'ShadowRootInit': The provided value '{mode}' is not a valid enum value of type ShadowRootMode."
            ),
        );
        return;
    }
    if element.shadow_root.is_some() {
        super::node::throw_dom_exception(
            scope,
            "NotSupportedError",
            "Failed to execute 'attachShadow' on 'Element': Shadow root cannot be created on a host which already hosts a shadow tree.",
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
    let slot_assignment = if slot_assignment_value.is_undefined() {
        "named".to_owned()
    } else {
        let Some(value) = crate::webidl::dom_string_with_context(
            scope,
            slot_assignment_value,
            "Failed to execute 'attachShadow' on 'Element': Failed to read the 'slotAssignment' property from 'ShadowRootInit'",
        ) else {
            return;
        };
        value
    };
    if !matches!(slot_assignment.as_str(), "named" | "manual") {
        crate::webidl::throw_type_error(
            scope,
            &format!(
                "Failed to execute 'attachShadow' on 'Element': Failed to read the 'slotAssignment' property from 'ShadowRootInit': The provided value '{slot_assignment}' is not a valid enum value of type SlotAssignmentMode."
            ),
        );
        return;
    }
    let delegates_focus =
        !delegates_focus_value.is_undefined() && delegates_focus_value.boolean_value(scope);
    let serializable =
        !serializable_value.is_undefined() && serializable_value.boolean_value(scope);
    let clonable = !clonable_value.is_undefined() && clonable_value.boolean_value(scope);
    let registry_is_null = registry_value.is_null();
    let registry = if registry_value.is_null_or_undefined() {
        None
    } else {
        let Ok(registry) = v8::Local::<v8::Object>::try_from(registry_value) else {
            crate::webidl::throw_type_error(
                scope,
                "Failed to execute 'attachShadow' on 'Element': Failed to read the 'customElementRegistry' property from 'ShadowRootInit': Failed to convert value to 'CustomElementRegistry'.",
            );
            return;
        };
        if !super::custom_element_registry::is_registry(scope, registry) {
            crate::webidl::throw_type_error(
                scope,
                "Failed to execute 'attachShadow' on 'Element': Failed to read the 'customElementRegistry' property from 'ShadowRootInit': Failed to convert value to 'CustomElementRegistry'.",
            );
            return;
        }
        Some(registry)
    };
    match super::shadow_root::create(
        scope,
        arguments.this(),
        mode,
        delegates_focus,
        slot_assignment,
        serializable,
        clonable,
        registry,
        registry_is_null,
    ) {
        Ok(shadow) => {
            super::element::set_shadow_root(scope, arguments.this(), shadow);
            result.set(shadow.into());
        }
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn raw_member<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<v8::Local<'s, v8::Value>> {
    object.get(scope, v8::String::new(scope, name)?.into())
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
