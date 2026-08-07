pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let function = crate::webidl::create_function(
        scope,
        "getComputedStyle",
        1,
        v8::ConstructorBehavior::Throw,
        get_computed_style,
    )?;
    let global = scope.get_current_context().global(scope);
    let key = crate::webidl::string(scope, "getComputedStyle")?;
    if global.define_own_property(
        scope,
        key.into(),
        function.into(),
        v8::PropertyAttribute::NONE,
    ) == Some(true)
    {
        Ok(())
    } else {
        Err("cannot define window.getComputedStyle".to_owned())
    }
}

fn get_computed_style(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'getComputedStyle' on 'Window': 1 argument required, but only 0 present.",
        );
        return;
    }
    let Ok(element) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'getComputedStyle' on 'Window': parameter 1 is not of type 'Element'.",
        );
        return;
    };
    if super::element::record(scope, element).is_none() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'getComputedStyle' on 'Window': parameter 1 is not of type 'Element'.",
        );
        return;
    }
    let declarations = computed_declarations(scope, element);
    match super::css_style_declaration::create(scope, &declarations, None, None) {
        Ok(style) => {
            super::css_style_declaration::mark_readonly(scope, style);
            result.set(style.into());
        }
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn inline_properties(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
) -> Vec<super::css_style_declaration::CssProperty> {
    let Some(key) = v8::String::new(scope, "style") else {
        return Vec::new();
    };
    element
        .get(scope, key.into())
        .and_then(|value| v8::Local::<v8::Object>::try_from(value).ok())
        .and_then(|style| super::css_style_declaration::properties(scope, style))
        .unwrap_or_default()
}

pub(crate) fn computed_declarations(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
) -> String {
    let mut resolved = cascaded_properties(scope, element);
    let mut output = String::new();
    for mut property in resolved.drain(..) {
        if let Some(value) =
            super::css_calculation::computed_system_color(&property.name, &property.source)
        {
            property.value = value.to_owned();
        } else if super::css_calculation::is_length_property(&property.name)
            && super::css_calculation::needs_computed_length_resolution(
                &property.name,
                &property.source,
            )
        {
            if let Some(value) = super::element_layout::resolve_css_length(
                scope,
                element,
                &property.name,
                &property.source,
            ) {
                property.value =
                    super::css_calculation::serialize_computed_length(&property.name, value);
            }
        } else if let Some(value) =
            super::css_calculation::computed_non_length(&property.name, &property.source)
        {
            property.value = value;
        }
        output.push_str(&property.name);
        output.push(':');
        output.push_str(&property.value);
        if !property.priority.is_empty() {
            output.push_str(" !important");
        }
        output.push(';');
    }
    output
}

fn cascaded_properties(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
) -> Vec<super::css_style_declaration::CssProperty> {
    let mut resolved = Vec::<super::css_style_declaration::CssProperty>::new();
    merge_properties(
        &mut resolved,
        super::css_style_declaration::parse_declarations(&user_agent_declarations(scope, element)),
    );
    if let Some(document) = super::node::owner_document(scope, element) {
        let mut sheets = super::document_style_sheets_property::sheets(scope, document);
        sheets.extend(super::document_adopted_style_sheets_property::sheets(
            scope, document,
        ));
        for sheet in sheets {
            if super::style_sheet::is_disabled(scope, sheet) {
                continue;
            }
            for rule in super::css_style_sheet::rule_objects(scope, sheet) {
                let rule = v8::Local::new(scope, &rule);
                let Some((selector, properties)) =
                    super::css_style_rule::selector_and_properties(scope, rule)
                else {
                    continue;
                };
                if !super::dom_selector::matches_selector(scope, element, &selector, document)
                    .unwrap_or(false)
                {
                    continue;
                }
                merge_properties(&mut resolved, properties);
            }
        }
    }
    merge_properties(&mut resolved, inline_properties(scope, element));
    resolved
}

pub(crate) fn cascaded_property_source(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<String> {
    cascaded_properties(scope, element)
        .into_iter()
        .find(|property| property.name == name)
        .map(|property| property.source)
}

fn user_agent_declarations(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
) -> String {
    let tag = super::element::record(scope, element)
        .map(|record| record.tag_name)
        .unwrap_or_default();
    let css = &crate::fingerprint::edge(scope).css;
    if tag.eq_ignore_ascii_case("BODY") {
        return css.body.clone();
    }
    if !tag.eq_ignore_ascii_case("INPUT") {
        return String::new();
    }
    let input_type = super::html_input_element::record(scope, element)
        .map(|record| record.input_type)
        .unwrap_or_else(|| "text".to_owned());
    let geometry = match input_type.as_str() {
        "hidden" => &css.input_hidden,
        "search" => &css.input_search,
        "checkbox" | "radio" => &css.input_checkbox_radio,
        "range" => &css.input_range,
        "color" => &css.input_color,
        "date" => &css.input_date,
        "time" => &css.input_time,
        "datetime-local" => &css.input_datetime_local,
        "month" => &css.input_month,
        "week" => &css.input_week,
        "image" => &css.input_image,
        "button" => &css.input_button,
        "submit" | "reset" => &css.input_submit_reset,
        "file" => &css.input_file,
        _ => &css.input_text,
    };
    if css.input_common.is_empty() {
        geometry.clone()
    } else {
        format!("{geometry};{}", css.input_common)
    }
}

pub(crate) fn computed_property_value(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    name: &str,
) -> String {
    super::css_style_declaration::parse_declarations(&computed_declarations(scope, element))
        .into_iter()
        .find(|property| property.name == name)
        .map(|property| property.value)
        .unwrap_or_default()
}

fn merge_properties(
    resolved: &mut Vec<super::css_style_declaration::CssProperty>,
    incoming: Vec<super::css_style_declaration::CssProperty>,
) {
    for mut property in incoming {
        property.value = normalize_computed_value(&property.name, &property.value);
        if let Some(current) = resolved
            .iter_mut()
            .find(|current| current.name == property.name)
        {
            if current.priority != "important" || property.priority == "important" {
                *current = property;
            }
        } else {
            resolved.push(property);
        }
    }
}

fn normalize_computed_value(name: &str, value: &str) -> String {
    if matches!(
        name,
        "color"
            | "background-color"
            | "border-top-color"
            | "border-right-color"
            | "border-bottom-color"
            | "border-left-color"
            | "outline-color"
            | "text-decoration-color"
    ) && ((value.starts_with("rgb(") && value.ends_with(')'))
        || (value.starts_with("rgba(") && value.ends_with(')')))
    {
        let open = value.find('(').unwrap_or(0);
        let function = &value[..open];
        let arguments = &value[open + 1..value.len() - 1];
        return format!(
            "{function}({})",
            arguments
                .split(',')
                .map(str::trim)
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    value.to_owned()
}
