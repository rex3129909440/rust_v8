const EDGE_INHERITED_DEFAULTS: &[(&str, &str)] = &[
    ("-webkit-text-fill-color", "currentcolor"),
    ("-webkit-text-stroke-color", "currentcolor"),
    ("-webkit-text-stroke-width", "0px"),
    ("border-collapse", "separate"),
    ("border-spacing", "0px 0px"),
    ("caption-side", "top"),
    ("color", "rgb(0, 0, 0)"),
    ("color-scheme", "normal"),
    ("cursor", "auto"),
    ("direction", "ltr"),
    ("empty-cells", "show"),
    ("fill", "rgb(0, 0, 0)"),
    ("font-family", "\"Times New Roman\""),
    ("font-feature-settings", "normal"),
    ("font-kerning", "auto"),
    ("font-optical-sizing", "auto"),
    ("font-size", "16px"),
    ("font-stretch", "100%"),
    ("font-style", "normal"),
    ("font-variant", "normal"),
    ("font-variant-caps", "normal"),
    ("font-variation-settings", "normal"),
    ("font-weight", "400"),
    ("hyphens", "manual"),
    ("image-rendering", "auto"),
    ("letter-spacing", "normal"),
    ("line-break", "auto"),
    ("line-height", "normal"),
    ("list-style-position", "outside"),
    ("list-style-type", "disc"),
    ("orphans", "2"),
    ("overflow-wrap", "normal"),
    ("pointer-events", "auto"),
    ("quotes", "auto"),
    ("ruby-position", "over"),
    ("stroke", "none"),
    ("tab-size", "8"),
    ("text-align", "start"),
    ("text-decoration-skip-ink", "auto"),
    ("text-emphasis-color", "currentcolor"),
    ("text-emphasis-position", "over"),
    ("text-indent", "0px"),
    ("text-rendering", "auto"),
    ("text-shadow", "none"),
    ("text-size-adjust", "auto"),
    ("text-transform", "none"),
    ("visibility", "visible"),
    ("white-space", "normal"),
    ("widows", "2"),
    ("word-break", "normal"),
    ("word-spacing", "0px"),
    ("writing-mode", "horizontal-tb"),
];

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
    let properties = computed_properties(scope, element);
    match super::css_style_declaration::create_readonly(scope, properties) {
        Ok(style) => result.set(style.into()),
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

fn computed_properties(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
) -> Vec<super::css_style_declaration::CssProperty> {
    // Chromium returns an empty resolved declaration for an element that is
    // not participating in a document tree, even when it has inline style.
    if !super::node::is_connected(scope, element) {
        return Vec::new();
    }
    let specified = specified_properties(scope, element);
    let specified_names = specified
        .iter()
        .map(|property| property.name.clone())
        .collect::<std::collections::HashSet<_>>();
    let overflow_axes = specified
        .iter()
        .find(|property| property.name == "overflow")
        .map(|property| {
            let values = property.value.split_whitespace().collect::<Vec<_>>();
            let x = values.first().copied().unwrap_or("visible").to_owned();
            let y = values
                .get(1)
                .copied()
                .or_else(|| values.first().copied())
                .unwrap_or("visible")
                .to_owned();
            (x, y)
        });
    let mut resolved = cascaded_properties(scope, element);
    let layout = super::element_layout::compute(scope, element);
    let current_color = computed_property_value(scope, element, "color");
    let horizontal_writing = computed_property_value(scope, element, "writing-mode")
        .eq_ignore_ascii_case("horizontal-tb");
    let mut output = Vec::with_capacity(resolved.len());
    for mut property in resolved.drain(..) {
        if let Some((x, y)) = &overflow_axes {
            let shorthand_value = match property.name.as_str() {
                "overflow-x" if !specified_names.contains("overflow-x") => Some(x),
                "overflow-y" if !specified_names.contains("overflow-y") => Some(y),
                _ => None,
            };
            if let Some(value) = shorthand_value {
                property.value = value.clone();
                property.source = value.clone();
            }
        }
        let preferences = &crate::fingerprint::edge(scope).media_preferences;
        let captured_default = !specified_names.contains(&property.name)
            && super::css_computed_style_initial_values::value(&property.name)
                .is_some_and(|value| value == property.source);
        if property.source.trim().eq_ignore_ascii_case("currentcolor") {
            property.value = current_color.clone();
        } else if !captured_default
            && let Some(value) = super::css_calculation::computed_color_with_preferences(
                &property.name,
                &property.source,
                preferences.forced_colors,
                &preferences.color_scheme,
            )
        {
            property.value = value;
        } else if !captured_default
            && super::css_calculation::is_length_property(&property.name)
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
        } else if !captured_default
            && let Some(value) =
                super::css_calculation::computed_non_length(&property.name, &property.source)
        {
            property.value = value;
        }
        if layout.rendered {
            let auto = property.source.trim().eq_ignore_ascii_case("auto");
            let used_length = match property.name.as_str() {
                "width" if auto => Some(layout.content_width),
                "height" if auto => Some(layout.content_height),
                "inline-size" if auto => Some(if horizontal_writing {
                    layout.content_width
                } else {
                    layout.content_height
                }),
                "block-size" if auto => Some(if horizontal_writing {
                    layout.content_height
                } else {
                    layout.content_width
                }),
                _ => None,
            };
            if let Some(value) = used_length {
                property.value =
                    super::css_calculation::serialize_computed_length(&property.name, value);
            } else if property.name == "transform-origin" && property.source == "50% 50% 0px" {
                property.value = format!(
                    "{} {}",
                    super::css_calculation::serialize_computed_length(
                        "width",
                        layout.content_width / 2.0,
                    ),
                    super::css_calculation::serialize_computed_length(
                        "height",
                        layout.content_height / 2.0,
                    )
                );
            } else if property.name == "perspective-origin" && property.source == "50% 50%" {
                property.value = format!(
                    "{} {}",
                    super::css_calculation::serialize_computed_length(
                        "width",
                        layout.content_width / 2.0,
                    ),
                    super::css_calculation::serialize_computed_length(
                        "height",
                        layout.content_height / 2.0,
                    )
                );
            }
        }
        output.push(property);
    }
    output
}

fn cascaded_properties(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
) -> Vec<super::css_style_declaration::CssProperty> {
    let mut resolved = specified_properties(scope, element);
    resolve_initial_and_inherited_properties(scope, element, &mut resolved);
    let mut present = resolved
        .iter()
        .map(|property| property.name.clone())
        .collect::<std::collections::HashSet<_>>();
    for (name, value) in super::css_computed_style_initial_values::EDGE_150_INITIAL_COMPUTED_VALUES
    {
        if !present.insert((*name).to_owned()) {
            continue;
        }
        resolved.push(super::css_style_declaration::CssProperty {
            name: (*name).to_owned(),
            value: (*value).to_owned(),
            priority: String::new(),
            source: (*value).to_owned(),
        });
    }
    resolve_computed_color(scope, element, &mut resolved);
    resolved
}

fn specified_properties(
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

fn resolve_initial_and_inherited_properties(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    resolved: &mut Vec<super::css_style_declaration::CssProperty>,
) {
    const INITIAL_DEFAULTS: &[(&str, &str)] =
        &[("position", "static"), ("box-sizing", "content-box")];

    let inherited_parent = nearest_element_parent(scope, element)
        .map(|parent| inherited_properties(scope, parent, EDGE_INHERITED_DEFAULTS));
    for (name, initial) in EDGE_INHERITED_DEFAULTS {
        let inherited = inherited_parent
            .as_ref()
            .and_then(|properties| properties.iter().find(|property| property.name == *name))
            .map(|property| property.value.clone())
            .unwrap_or_else(|| (*initial).to_owned());
        if let Some(property) = resolved.iter_mut().find(|property| property.name == *name) {
            let keyword = property.source.trim().to_ascii_lowercase();
            if matches!(keyword.as_str(), "inherit" | "unset")
                || *name == "color" && keyword == "currentcolor"
            {
                property.value = inherited.clone();
                property.source = inherited;
            } else if matches!(keyword.as_str(), "initial" | "revert" | "revert-layer") {
                property.value = (*initial).to_owned();
                property.source = (*initial).to_owned();
            }
        } else {
            resolved.push(super::css_style_declaration::CssProperty {
                name: (*name).to_owned(),
                value: inherited.clone(),
                priority: String::new(),
                source: inherited,
            });
        }
    }
    for (name, initial) in INITIAL_DEFAULTS {
        if resolved.iter().any(|property| property.name == *name) {
            continue;
        }
        resolved.push(super::css_style_declaration::CssProperty {
            name: (*name).to_owned(),
            value: (*initial).to_owned(),
            priority: String::new(),
            source: (*initial).to_owned(),
        });
    }
}

fn inherited_properties(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    defaults: &[(&str, &str)],
) -> Vec<super::css_style_declaration::CssProperty> {
    let specified = specified_properties(scope, element);
    let parent = nearest_element_parent(scope, element)
        .map(|parent| inherited_properties(scope, parent, defaults));
    defaults
        .iter()
        .map(|(name, initial)| {
            let inherited = parent
                .as_ref()
                .and_then(|properties| properties.iter().find(|property| property.name == *name))
                .map(|property| property.value.clone())
                .unwrap_or_else(|| (*initial).to_owned());
            let own = specified.iter().find(|property| property.name == *name);
            let value = own
                .filter(|property| {
                    let keyword = property.source.trim().to_ascii_lowercase();
                    !matches!(keyword.as_str(), "inherit" | "unset")
                        && !(*name == "color" && keyword == "currentcolor")
                })
                .map(|property| {
                    let keyword = property.source.trim().to_ascii_lowercase();
                    if matches!(keyword.as_str(), "initial" | "revert" | "revert-layer") {
                        (*initial).to_owned()
                    } else {
                        property.value.clone()
                    }
                })
                .unwrap_or(inherited);
            super::css_style_declaration::CssProperty {
                name: (*name).to_owned(),
                value: value.clone(),
                priority: String::new(),
                source: value,
            }
        })
        .collect()
}

fn resolve_computed_color(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    resolved: &mut Vec<super::css_style_declaration::CssProperty>,
) {
    let color_index = resolved
        .iter()
        .position(|property| property.name == "color");
    let inherited = color_index.is_none_or(|index| {
        matches!(
            resolved[index].source.trim().to_ascii_lowercase().as_str(),
            "inherit" | "unset" | "currentcolor"
        )
    });
    let initial = color_index.is_some_and(|index| {
        matches!(
            resolved[index].source.trim().to_ascii_lowercase().as_str(),
            "initial" | "revert" | "revert-layer"
        )
    });
    if !inherited && !initial {
        return;
    }

    let inherited_property = inherited
        .then(|| nearest_element_parent(scope, element))
        .flatten()
        .and_then(|parent| {
            cascaded_properties(scope, parent)
                .into_iter()
                .find(|property| property.name == "color")
        });
    let replacement =
        inherited_property.unwrap_or_else(|| super::css_style_declaration::CssProperty {
            name: "color".to_owned(),
            value: "rgb(0, 0, 0)".to_owned(),
            priority: String::new(),
            source: "rgb(0, 0, 0)".to_owned(),
        });
    if let Some(index) = color_index {
        resolved[index] = replacement;
    } else {
        resolved.push(replacement);
    }
}

fn nearest_element_parent<'s>(
    scope: &v8::PinScope<'s, '_>,
    element: v8::Local<'s, v8::Object>,
) -> Option<v8::Local<'s, v8::Object>> {
    let mut parent = super::node::parent(scope, element);
    while let Some(candidate) = parent {
        if super::element::record(scope, candidate).is_some() {
            return Some(candidate);
        }
        parent = super::node::parent(scope, candidate);
    }
    None
}

pub(crate) fn cascaded_property_source(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<String> {
    if let Some(source) = specified_properties(scope, element)
        .into_iter()
        .find(|property| property.name == name)
        .map(|property| property.source)
    {
        return Some(source);
    }
    inherited_initial_value(name).map(|initial| {
        nearest_element_parent(scope, element)
            .and_then(|parent| cascaded_property_source(scope, parent, name))
            .unwrap_or_else(|| initial.to_owned())
    })
}

pub(crate) fn own_specified_property_source(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<String> {
    specified_properties(scope, element)
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
    let mut declarations = if tag.eq_ignore_ascii_case("BODY") {
        css.body.clone()
    } else if tag.eq_ignore_ascii_case("BUTTON") {
        // Edge 150's Windows UA sheet gives a button its own control font and
        // intrinsic inline-block box instead of inheriting the document's
        // Times New Roman defaults. These declarations are observable through
        // getComputedStyle and are also consumed by the layout layer.
        "display:inline-block;box-sizing:border-box;height:21px;font-family:Arial;font-size:13.3333px;font-stretch:100%;font-style:normal;font-variant:normal;font-weight:400;line-height:normal;padding:1px 6px;border:2px outset rgb(0, 0, 0)".to_owned()
    } else if tag.eq_ignore_ascii_case("INPUT") {
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
    } else if tag.eq_ignore_ascii_case("RT") {
        "display:ruby-text;font-size:50%".to_owned()
    } else {
        default_display_for_tag(&tag)
            .map(|display| format!("display:{display}"))
            .unwrap_or_default()
    };
    if super::element::attribute_value(scope, element, "hidden")
        .is_some_and(|value| !value.eq_ignore_ascii_case("until-found"))
    {
        if !declarations.is_empty() {
            declarations.push(';');
        }
        declarations.push_str("display:none !important");
    }
    declarations
}

fn default_display_for_tag(tag: &str) -> Option<&'static str> {
    Some(match tag {
        "AREA" | "BASE" | "BASEFONT" | "DATALIST" | "HEAD" | "LINK" | "META" | "NOFRAMES"
        | "PARAM" | "SCRIPT" | "STYLE" | "TEMPLATE" | "TITLE" => "none",
        "ADDRESS" | "ARTICLE" | "ASIDE" | "BLOCKQUOTE" | "BODY" | "DD" | "DIV" | "DL" | "DT"
        | "FIELDSET" | "FIGCAPTION" | "FIGURE" | "FOOTER" | "FORM" | "H1" | "H2" | "H3" | "H4"
        | "H5" | "H6" | "HEADER" | "HGROUP" | "HTML" | "MAIN" | "NAV" | "OL" | "P" | "PRE"
        | "SEARCH" | "SECTION" | "UL" => "block",
        "DETAILS" => "block",
        "SUMMARY" | "LI" => "list-item",
        "TABLE" => "table",
        "CAPTION" => "table-caption",
        "COLGROUP" => "table-column-group",
        "COL" => "table-column",
        "THEAD" => "table-header-group",
        "TBODY" => "table-row-group",
        "TFOOT" => "table-footer-group",
        "TR" => "table-row",
        "TD" | "TH" => "table-cell",
        "BUTTON" | "METER" | "PROGRESS" | "SELECT" | "TEXTAREA" => "inline-block",
        "HR" => "block",
        "RB" => "ruby-base",
        "RP" => "none",
        "RT" => "ruby-text",
        "RTC" => "ruby-text-container",
        "RUBY" => "ruby",
        "CANVAS" | "IFRAME" | "IMG" | "OBJECT" | "VIDEO" => "inline",
        "A" | "ABBR" | "B" | "BDI" | "BDO" | "BIG" | "BR" | "CITE" | "CODE" | "DATA" | "DEL"
        | "DFN" | "EM" | "FONT" | "I" | "INS" | "KBD" | "LABEL" | "LEGEND" | "MARK" | "OUTPUT"
        | "PICTURE" | "Q" | "S" | "SAMP" | "SMALL" | "SPAN" | "STRIKE" | "STRONG" | "SUB"
        | "SUP" | "TIME" | "TT" | "U" | "VAR" | "WBR" => "inline",
        _ => return None,
    })
}

pub(crate) fn computed_property_value(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    name: &str,
) -> String {
    if name == "font" {
        let properties = [
            "font-family",
            "font-size",
            "font-stretch",
            "font-style",
            "font-variant",
            "font-weight",
            "line-height",
        ]
        .into_iter()
        .map(|property_name| super::css_style_declaration::CssProperty {
            name: property_name.to_owned(),
            value: computed_property_value(scope, element, property_name),
            priority: String::new(),
            source: String::new(),
        })
        .collect::<Vec<_>>();
        return super::css_style_declaration::computed_value_from_properties(&properties, name);
    }
    let mut property = specified_properties(scope, element)
        .into_iter()
        .find(|property| property.name == name);
    if property.is_none() {
        if let Some(initial) = inherited_initial_value(name) {
            return nearest_element_parent(scope, element)
                .map(|parent| computed_property_value(scope, parent, name))
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| initial.to_owned());
        }
        return non_inherited_initial_value(name)
            .or_else(|| super::css_computed_style_initial_values::value(name))
            .unwrap_or_default()
            .to_owned();
    }
    let mut property = property.take().expect("computed property");
    if inherited_initial_value(name).is_some()
        && matches!(
            property.source.trim().to_ascii_lowercase().as_str(),
            "inherit" | "unset"
        )
    {
        return nearest_element_parent(scope, element)
            .map(|parent| computed_property_value(scope, parent, name))
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| inherited_initial_value(name).unwrap_or("").to_owned());
    }
    if matches!(
        property.source.trim().to_ascii_lowercase().as_str(),
        "initial" | "revert" | "revert-layer"
    ) {
        return inherited_initial_value(name)
            .or_else(|| non_inherited_initial_value(name))
            .unwrap_or_default()
            .to_owned();
    }
    let preferences = &crate::fingerprint::edge(scope).media_preferences;
    if let Some(value) = super::css_calculation::computed_color_with_preferences(
        &property.name,
        &property.source,
        preferences.forced_colors,
        &preferences.color_scheme,
    ) {
        property.value = value;
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
    property.value
}

fn inherited_initial_value(name: &str) -> Option<&'static str> {
    EDGE_INHERITED_DEFAULTS
        .iter()
        .find_map(|(property, initial)| (*property == name).then_some(*initial))
}

fn non_inherited_initial_value(name: &str) -> Option<&'static str> {
    match name {
        "position" => Some("static"),
        "box-sizing" => Some("content-box"),
        _ => None,
    }
}

fn merge_properties(
    resolved: &mut Vec<super::css_style_declaration::CssProperty>,
    incoming: Vec<super::css_style_declaration::CssProperty>,
) {
    for property in incoming {
        let font_longhands = (property.name == "font").then(|| {
            super::css_style_declaration::font_longhands_from_shorthand(&property.source)
                .into_iter()
                .map(|mut longhand| {
                    longhand.priority = property.priority.clone();
                    longhand
                })
                .collect::<Vec<_>>()
        });
        merge_one_property(resolved, property);
        if let Some(longhands) = font_longhands {
            for longhand in longhands {
                merge_one_property(resolved, longhand);
            }
        }
    }
}

fn merge_one_property(
    resolved: &mut Vec<super::css_style_declaration::CssProperty>,
    mut property: super::css_style_declaration::CssProperty,
) {
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
