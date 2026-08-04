use std::collections::HashMap;

#[derive(Clone)]
struct CssStyleSheetRecord {
    owner_rule: Option<v8::Global<v8::Object>>,
    rule_list: v8::Global<v8::Object>,
    constructed: bool,
}

#[derive(Default)]
pub(crate) struct CssStyleSheetStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, CssStyleSheetRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CssStyleSheetStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "CSSStyleSheet", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<CssStyleSheetStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "CSSStyleSheet",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "ownerRule", get_owner_rule)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "cssRules", get_css_rules)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "rules", get_rules)?;
    crate::webidl::define_method(scope, prototype, "addRule", 0, add_rule)?;
    crate::webidl::define_method(scope, prototype, "deleteRule", 1, delete_rule)?;
    crate::webidl::define_method(scope, prototype, "insertRule", 1, insert_rule)?;
    crate::webidl::define_method(scope, prototype, "removeRule", 0, remove_rule)?;
    crate::webidl::define_method(scope, prototype, "replace", 1, replace)?;
    crate::webidl::define_method(scope, prototype, "replaceSync", 1, replace_sync)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::style_sheet::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<CssStyleSheetStore>()
        .ok_or_else(|| "CSSStyleSheet state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn option<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<v8::Local<'s, v8::Value>> {
    let key = v8::String::new(scope, name)?;
    object.get(scope, key.into())
}

fn attach(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    media_text: &str,
    disabled: bool,
) -> Result<(), String> {
    let media = super::media_list::create(scope, media_text)?;
    super::style_sheet::attach(scope, object, None, None, media.into(), disabled);
    let rule_list = super::css_rule_list::create(scope, Vec::new())?;
    let rule_list = v8::Global::new(scope, rule_list);
    scope
        .get_slot_mut::<CssStyleSheetStore>()
        .ok_or_else(|| "CSSStyleSheet state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            CssStyleSheetRecord {
                owner_rule: None,
                rule_list,
                constructed: true,
            },
        );
    Ok(())
}

pub(crate) fn create_for_owner<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    owner: v8::Local<'_, v8::Object>,
    href: Option<String>,
    media_text: &str,
    disabled: bool,
    text: &str,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create owner-backed CSSStyleSheet".to_owned());
    }
    let media = super::media_list::create(scope, media_text)?;
    super::style_sheet::attach(scope, object, href, None, media.into(), disabled);
    if !super::style_sheet::set_owner_node(scope, object, owner) {
        return Err("cannot attach stylesheet owner node".to_owned());
    }
    let rule_list = super::css_rule_list::create(scope, Vec::new())?;
    let rule_list = v8::Global::new(scope, rule_list);
    scope
        .get_slot_mut::<CssStyleSheetStore>()
        .ok_or_else(|| "CSSStyleSheet state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            CssStyleSheetRecord {
                owner_rule: None,
                rule_list,
                constructed: false,
            },
        );
    replace_rules(scope, object, text)?;
    Ok(object)
}

pub(crate) fn is_css_style_sheet(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> bool {
    record(scope, object).is_some()
}

pub(crate) fn is_constructed(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> bool {
    record(scope, object).is_some_and(|record| record.constructed)
}

pub(crate) fn rule_objects(
    scope: &v8::PinScope<'_, '_>,
    sheet: v8::Local<'_, v8::Object>,
) -> Vec<v8::Global<v8::Object>> {
    let Some(record) = record(scope, sheet) else {
        return Vec::new();
    };
    let list = v8::Local::new(scope, &record.rule_list);
    super::css_rule_list::rules(scope, list).unwrap_or_default()
}

pub(crate) fn create_imported<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    href: String,
    media_text: &str,
    owner_rule: v8::Local<'_, v8::Object>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create imported CSSStyleSheet".to_owned());
    }
    let media = super::media_list::create(scope, media_text)?;
    super::style_sheet::attach(scope, object, Some(href), None, media.into(), false);
    let rule_list = super::css_rule_list::create(scope, Vec::new())?;
    let owner_rule = v8::Global::new(scope, owner_rule);
    let rule_list = v8::Global::new(scope, rule_list);
    scope
        .get_slot_mut::<CssStyleSheetStore>()
        .ok_or_else(|| "CSSStyleSheet state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            CssStyleSheetRecord {
                owner_rule: Some(owner_rule),
                rule_list,
                constructed: false,
            },
        );
    Ok(object)
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(scope, "CSSStyleSheet must be constructed");
        return;
    }
    let options = v8::Local::<v8::Object>::try_from(arguments.get(0)).ok();
    let media = options
        .and_then(|options| option(scope, options, "media"))
        .filter(|value| !value.is_undefined())
        .map(|value| crate::webidl::value_to_string(scope, value))
        .unwrap_or_default();
    let disabled = options
        .and_then(|options| option(scope, options, "disabled"))
        .is_some_and(|value| value.boolean_value(scope));
    match attach(scope, arguments.this(), &media, disabled) {
        Ok(()) => result.set(arguments.this().into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<CssStyleSheetRecord> {
    scope
        .get_slot::<CssStyleSheetStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_owner_rule(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match record(scope, arguments.this()) {
        Some(record) => match record.owner_rule {
            Some(rule) => result.set(v8::Local::new(scope, &rule).into()),
            None => result.set(v8::null(scope).into()),
        },
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}

fn return_rules(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.rule_list).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_css_rules(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    return_rules(scope, arguments, result);
}

fn get_rules(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    return_rules(scope, arguments, result);
}

#[derive(Clone)]
struct ParsedRule {
    header: String,
    body: String,
}

fn split_rules(text: &str) -> Result<Vec<ParsedRule>, String> {
    let mut rules = Vec::new();
    let mut cursor = 0;
    let bytes = text.as_bytes();
    while cursor < bytes.len() {
        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if cursor >= bytes.len() {
            break;
        }
        if text[cursor..].to_ascii_lowercase().starts_with("@import") {
            let Some(relative_end) = text[cursor..].find(';') else {
                return Err("CSS import rule is missing ';'".to_owned());
            };
            let end = cursor + relative_end;
            rules.push(ParsedRule {
                header: text[cursor..end].trim().to_owned(),
                body: String::new(),
            });
            cursor = end + 1;
            continue;
        }
        if text[cursor..]
            .to_ascii_lowercase()
            .starts_with("@namespace")
        {
            let Some(relative_end) = text[cursor..].find(';') else {
                return Err("CSS namespace rule is missing ';'".to_owned());
            };
            let end = cursor + relative_end;
            rules.push(ParsedRule {
                header: text[cursor..end].trim().to_owned(),
                body: String::new(),
            });
            cursor = end + 1;
            continue;
        }
        if text[cursor..].to_ascii_lowercase().starts_with("@layer") {
            let relative_semicolon = text[cursor..].find(';');
            let relative_brace = text[cursor..].find('{');
            let is_statement = match (relative_semicolon, relative_brace) {
                (Some(semicolon), Some(brace)) => semicolon < brace,
                (Some(_), None) => true,
                _ => false,
            };
            if is_statement {
                let end = cursor + relative_semicolon.expect("layer semicolon");
                rules.push(ParsedRule {
                    header: text[cursor..=end].trim().to_owned(),
                    body: String::new(),
                });
                cursor = end + 1;
                continue;
            }
        }
        let header_start = cursor;
        let mut quote = None;
        while cursor < bytes.len() {
            let character = bytes[cursor] as char;
            if let Some(current_quote) = quote {
                if character == current_quote && (cursor == 0 || bytes[cursor - 1] != b'\\') {
                    quote = None;
                }
            } else if character == '\'' || character == '"' {
                quote = Some(character);
            } else if character == '{' {
                break;
            }
            cursor += 1;
        }
        if cursor >= bytes.len() {
            return Err("CSS rule is missing a block".to_owned());
        }
        let header = text[header_start..cursor].trim().to_owned();
        cursor += 1;
        let body_start = cursor;
        let mut depth = 1_u32;
        quote = None;
        while cursor < bytes.len() && depth > 0 {
            let character = bytes[cursor] as char;
            if let Some(current_quote) = quote {
                if character == current_quote && bytes[cursor - 1] != b'\\' {
                    quote = None;
                }
            } else if character == '\'' || character == '"' {
                quote = Some(character);
            } else if character == '{' {
                depth += 1;
            } else if character == '}' {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            cursor += 1;
        }
        if depth != 0 {
            return Err("CSS rule has an unterminated block".to_owned());
        }
        let body = text[body_start..cursor].trim().to_owned();
        cursor += 1;
        if header.is_empty() {
            return Err("CSS rule has no selector".to_owned());
        }
        rules.push(ParsedRule { header, body });
    }
    Ok(rules)
}

fn create_parsed_rule<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    parsed: ParsedRule,
    parent_style_sheet: Option<v8::Local<'_, v8::Object>>,
    parent_rule: Option<v8::Local<'_, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let lower = parsed.header.to_ascii_lowercase();
    if lower.starts_with("@keyframes") {
        let name = parsed.header["@keyframes".len()..].trim().to_owned();
        super::css_keyframes_rule::create(
            scope,
            name,
            &parsed.body,
            parent_style_sheet,
            parent_rule,
        )
    } else if lower.starts_with("@function") {
        super::css_function_rule::create(
            scope,
            &parsed.header,
            &parsed.body,
            parent_style_sheet,
            parent_rule,
        )
    } else if lower.starts_with("@font-feature-values") {
        let family = parsed.header["@font-feature-values".len()..]
            .trim()
            .to_owned();
        super::css_font_feature_values_rule::create(
            scope,
            family,
            &parsed.body,
            parent_style_sheet,
            parent_rule,
        )
    } else if lower.starts_with("@import") {
        let prelude = parsed.header["@import".len()..].trim();
        super::css_import_rule::create(scope, prelude, parent_style_sheet, parent_rule)
    } else if lower.starts_with("@font-palette-values") {
        let name = parsed.header["@font-palette-values".len()..]
            .trim()
            .to_owned();
        super::css_font_palette_values_rule::create(
            scope,
            name,
            &parsed.body,
            parent_style_sheet,
            parent_rule,
        )
    } else if lower == "@font-face" {
        super::css_font_face_rule::create(scope, &parsed.body, parent_style_sheet, parent_rule)
    } else if lower.starts_with("@counter-style") {
        let name = parsed.header["@counter-style".len()..].trim().to_owned();
        super::css_counter_style_rule::create(
            scope,
            name,
            &parsed.body,
            parent_style_sheet,
            parent_rule,
        )
    } else if lower.starts_with("@container") {
        let condition = parsed.header["@container".len()..].trim().to_owned();
        super::css_container_rule::create(
            scope,
            condition,
            &parsed.body,
            parent_style_sheet,
            parent_rule,
        )
    } else if lower.starts_with("@scope") {
        let prelude = parsed.header["@scope".len()..].trim();
        let (start, end) = if let Some((start, end)) = prelude.split_once(" to ") {
            (scoped_selector(start), scoped_selector(end))
        } else {
            (scoped_selector(prelude), None)
        };
        super::css_scope_rule::create(
            scope,
            start,
            end,
            &parsed.body,
            parent_style_sheet,
            parent_rule,
        )
    } else if lower.starts_with("@media") {
        let condition = parsed.header["@media".len()..].trim().to_owned();
        super::css_media_rule::create(
            scope,
            condition,
            &parsed.body,
            parent_style_sheet,
            parent_rule,
        )
    } else if lower.starts_with("@supports") {
        let condition = parsed.header["@supports".len()..].trim().to_owned();
        super::css_supports_rule::create(
            scope,
            condition,
            &parsed.body,
            parent_style_sheet,
            parent_rule,
        )
    } else if lower.starts_with("@page") {
        let selector = parsed.header["@page".len()..].trim().to_owned();
        super::css_page_rule::create(
            scope,
            selector,
            &parsed.body,
            parent_style_sheet,
            parent_rule,
        )
    } else if lower == "@starting-style" {
        super::css_starting_style_rule::create(scope, &parsed.body, parent_style_sheet, parent_rule)
    } else if lower.starts_with("@property") {
        let name = parsed.header["@property".len()..].trim().to_owned();
        super::css_property_rule::create(scope, name, &parsed.body, parent_style_sheet, parent_rule)
    } else if lower.starts_with("@position-try") {
        let name = parsed.header["@position-try".len()..].trim().to_owned();
        super::css_position_try_rule::create(
            scope,
            name,
            &parsed.body,
            parent_style_sheet,
            parent_rule,
        )
    } else if lower.starts_with("@namespace") {
        let (prefix, uri) = namespace_parts(&parsed.header)?;
        super::css_namespace_rule::create(scope, prefix, uri, parent_style_sheet)
    } else if lower.starts_with("@layer") {
        let statement = parsed.header.ends_with(';');
        let prelude = parsed.header["@layer".len()..]
            .trim()
            .trim_end_matches(';')
            .trim();
        if statement {
            let names = prelude
                .split(',')
                .map(str::trim)
                .filter(|name| !name.is_empty())
                .map(str::to_owned)
                .collect();
            super::css_layer_statement_rule::create(scope, names, parent_style_sheet, parent_rule)
        } else {
            super::css_layer_block_rule::create(
                scope,
                prelude.to_owned(),
                &parsed.body,
                parent_style_sheet,
                parent_rule,
            )
        }
    } else if is_page_margin_header(&lower) {
        let name = parsed.header.trim().trim_start_matches('@').to_owned();
        super::css_margin_rule::create(scope, name, &parsed.body, parent_style_sheet, parent_rule)
    } else if parsed.header.starts_with('@') {
        Err("Unsupported CSS at-rule".to_owned())
    } else {
        super::css_style_rule::create(
            scope,
            parsed.header,
            parsed.body,
            parent_style_sheet,
            parent_rule,
        )
    }
}

fn is_page_margin_header(header: &str) -> bool {
    matches!(
        header,
        "@top-left-corner"
            | "@top-left"
            | "@top-center"
            | "@top-right"
            | "@top-right-corner"
            | "@bottom-left-corner"
            | "@bottom-left"
            | "@bottom-center"
            | "@bottom-right"
            | "@bottom-right-corner"
            | "@left-top"
            | "@left-middle"
            | "@left-bottom"
            | "@right-top"
            | "@right-middle"
            | "@right-bottom"
    )
}

fn namespace_parts(header: &str) -> Result<(String, String), String> {
    let rest = header["@namespace".len()..].trim();
    let lower = rest.to_ascii_lowercase();
    let url_start = lower
        .find("url(")
        .ok_or_else(|| "Namespace rule requires url()".to_owned())?;
    let prefix = rest[..url_start].trim().to_owned();
    let url = rest[url_start + 4..]
        .trim()
        .strip_suffix(')')
        .ok_or_else(|| "Namespace url() is not closed".to_owned())?
        .trim()
        .trim_matches(['\'', '"'])
        .to_owned();
    if url.is_empty() {
        Err("Namespace URI cannot be empty".to_owned())
    } else {
        Ok((prefix, url))
    }
}

fn scoped_selector(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    let value = value
        .strip_prefix('(')
        .and_then(|value| value.strip_suffix(')'))
        .unwrap_or(value)
        .trim();
    (!value.is_empty()).then(|| value.to_owned())
}

pub(crate) fn parse_rules<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    text: &str,
    parent_style_sheet: Option<v8::Local<'_, v8::Object>>,
    parent_rule: Option<v8::Local<'_, v8::Object>>,
) -> Result<Vec<v8::Local<'s, v8::Object>>, String> {
    let parsed = split_rules(text)?;
    let mut output = Vec::with_capacity(parsed.len());
    for rule in parsed {
        output.push(create_parsed_rule(
            scope,
            rule,
            parent_style_sheet,
            parent_rule,
        )?);
    }
    Ok(output)
}

pub(crate) fn parse_single_rule<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    text: &str,
    parent_style_sheet: Option<v8::Local<'_, v8::Object>>,
    parent_rule: Option<v8::Local<'_, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let mut parsed = split_rules(text)?;
    if parsed.len() != 1 {
        return Err("insertRule accepts exactly one CSS rule".to_owned());
    }
    create_parsed_rule(scope, parsed.remove(0), parent_style_sheet, parent_rule)
}

fn insert_rule(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let text = crate::webidl::value_to_string(scope, arguments.get(0));
    let list = v8::Local::new(scope, &record.rule_list);
    let length = super::css_rule_list::rules(scope, list)
        .map(|rules| rules.len())
        .unwrap_or(0);
    let index = if arguments.get(1).is_undefined() {
        0
    } else {
        arguments.get(1).uint32_value(scope).unwrap_or(u32::MAX) as usize
    };
    if index > length {
        super::node::throw_dom_exception(
            scope,
            "IndexSizeError",
            "The index is larger than the maximum stylesheet rule index.",
        );
        return;
    }
    let rule = match parse_single_rule(scope, &text, Some(arguments.this()), None) {
        Ok(rule) => rule,
        Err(message) => {
            super::node::throw_dom_exception(scope, "SyntaxError", &message);
            return;
        }
    };
    if super::css_rule_list::insert(scope, list, index, rule) {
        result.set(v8::Integer::new_from_unsigned(scope, index as u32).into());
    }
}

fn delete_at(scope: &mut v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>, index: usize) {
    let Some(record) = record(scope, object) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let list = v8::Local::new(scope, &record.rule_list);
    if !super::css_rule_list::delete(scope, list, index) {
        super::node::throw_dom_exception(
            scope,
            "IndexSizeError",
            "The index is not in the stylesheet rule list.",
        );
    }
}

fn delete_rule(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let index = arguments.get(0).uint32_value(scope).unwrap_or(u32::MAX) as usize;
    delete_at(scope, arguments.this(), index);
}

fn remove_rule(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let index = if arguments.get(0).is_undefined() {
        0
    } else {
        arguments.get(0).uint32_value(scope).unwrap_or(u32::MAX) as usize
    };
    delete_at(scope, arguments.this(), index);
}

fn add_rule(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let selector = crate::webidl::value_to_string(scope, arguments.get(0));
    let declarations = crate::webidl::value_to_string(scope, arguments.get(1));
    let text = format!("{selector} {{ {declarations} }}");
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let list = v8::Local::new(scope, &record.rule_list);
    let length = super::css_rule_list::rules(scope, list)
        .map(|rules| rules.len())
        .unwrap_or(0);
    let index = if arguments.get(2).is_undefined() {
        length
    } else {
        arguments
            .get(2)
            .uint32_value(scope)
            .unwrap_or(length as u32) as usize
    };
    let rule = match parse_single_rule(scope, &text, Some(arguments.this()), None) {
        Ok(rule) => rule,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    if super::css_rule_list::insert(scope, list, index, rule) {
        result.set(v8::Integer::new(scope, -1).into());
    }
}

pub(crate) fn replace_rules(
    scope: &mut v8::PinScope<'_, '_>,
    sheet: v8::Local<'_, v8::Object>,
    text: &str,
) -> Result<(), String> {
    let record = record(scope, sheet).ok_or_else(|| "Illegal invocation".to_owned())?;
    let rules = parse_rules(scope, text, Some(sheet), None)?;
    let list = v8::Local::new(scope, &record.rule_list);
    if super::css_rule_list::replace(scope, list, rules) {
        Ok(())
    } else {
        Err("cannot replace stylesheet rules".to_owned())
    }
}

fn replace_sync(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if !is_constructed(scope, arguments.this()) {
        super::node::throw_dom_exception(
            scope,
            "NotAllowedError",
            "Can't call replaceSync on non-constructed CSSStyleSheets.",
        );
        return;
    }
    let text = crate::webidl::value_to_string(scope, arguments.get(0));
    if let Err(message) = replace_rules(scope, arguments.this(), &text) {
        crate::webidl::throw_type_error(scope, &message);
    }
}

fn replace(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(resolver) = v8::PromiseResolver::new(scope) else {
        crate::webidl::throw_type_error(scope, "cannot create Promise");
        return;
    };
    if !is_constructed(scope, arguments.this()) {
        let exception = super::dom_exception::create(
            scope,
            "Can't call replace on non-constructed CSSStyleSheets.".to_owned(),
            "NotAllowedError".to_owned(),
        );
        match exception {
            Ok(exception) => {
                let _ = resolver.reject(scope, exception.into());
                result.set(resolver.get_promise(scope).into());
            }
            Err(message) => crate::webidl::throw_type_error(scope, &message),
        }
        return;
    }
    let text = crate::webidl::value_to_string(scope, arguments.get(0));
    if let Err(message) = replace_rules(scope, arguments.this(), &text) {
        crate::webidl::throw_type_error(scope, &message);
        return;
    }
    let _ = resolver.resolve(scope, arguments.this().into());
    result.set(resolver.get_promise(scope).into());
}
