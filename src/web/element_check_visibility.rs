pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "checkVisibility", 0, check_visibility)
}

fn check_visibility(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !super::element_method_support::ensure(scope, arguments.this()) {
        return;
    }
    let options = v8::Local::<v8::Object>::try_from(arguments.get(0)).ok();
    let check_opacity =
        option(scope, options, "opacityProperty") || option(scope, options, "checkOpacity");
    let check_visibility =
        option(scope, options, "visibilityProperty") || option(scope, options, "checkVisibility");
    let check_content_auto = option(scope, options, "contentVisibilityAuto");
    let visible = has_css_box(
        scope,
        arguments.this(),
        check_opacity,
        check_visibility,
        check_content_auto,
    );
    result.set(v8::Boolean::new(scope, visible).into());
}

fn option(
    scope: &v8::PinScope<'_, '_>,
    options: Option<v8::Local<'_, v8::Object>>,
    name: &str,
) -> bool {
    let Some(options) = options else {
        return false;
    };
    v8::String::new(scope, name)
        .and_then(|key| options.get(scope, key.into()))
        .is_some_and(|value| value.boolean_value(scope))
}

fn has_css_box(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    check_opacity: bool,
    check_visibility: bool,
    check_content_auto: bool,
) -> bool {
    if !super::node::is_connected(scope, element) {
        return false;
    }
    let target_id = element.get_identity_hash().get();
    let mut current = Some(element);
    while let Some(candidate) = current {
        if super::element::record(scope, candidate).is_some() {
            if super::element::attribute_value(scope, candidate, "hidden").is_some() {
                return false;
            }
            let display = style_value(scope, candidate, "display");
            if display == "none"
                || (candidate.get_identity_hash().get() == target_id && display == "contents")
            {
                return false;
            }
            let content_visibility = style_value(scope, candidate, "content-visibility");
            if content_visibility == "hidden" {
                return false;
            }
            if check_content_auto
                && content_visibility == "auto"
                && !super::node::is_connected(scope, candidate)
            {
                return false;
            }
            if check_visibility {
                let visibility = style_value(scope, candidate, "visibility");
                if visibility == "hidden" || visibility == "collapse" {
                    return false;
                }
            }
            if check_opacity {
                let opacity = style_value(scope, candidate, "opacity");
                if opacity_is_zero(&opacity) {
                    return false;
                }
            }
        }
        current = super::node::parent(scope, candidate)
            .or_else(|| super::shadow_root::host(scope, candidate));
    }
    true
}

fn style_value(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    name: &str,
) -> String {
    if let Some(record) = super::html_element::record(scope, element) {
        let style = v8::Local::new(scope, &record.style);
        if let Some(value) = super::css_style_declaration::property_value(scope, style, name)
            && !value.is_empty()
        {
            return normalize(value);
        }
    }
    let Some(style) = super::element::attribute_value(scope, element, "style") else {
        return String::new();
    };
    style
        .split(';')
        .filter_map(|declaration| declaration.split_once(':'))
        .find(|(property, _)| property.trim().eq_ignore_ascii_case(name))
        .map(|(_, value)| normalize(value.to_owned()))
        .unwrap_or_default()
}

fn normalize(value: String) -> String {
    value
        .trim()
        .strip_suffix("!important")
        .unwrap_or(value.trim())
        .trim()
        .to_ascii_lowercase()
}

fn opacity_is_zero(value: &str) -> bool {
    if let Some(percent) = value.strip_suffix('%') {
        return percent
            .trim()
            .parse::<f64>()
            .is_ok_and(|value| value == 0.0);
    }
    value.parse::<f64>().is_ok_and(|value| value == 0.0)
}
