use std::collections::HashMap;

#[derive(Clone)]
struct CssImportRuleRecord {
    href: String,
    media: v8::Global<v8::Object>,
    style_sheet: v8::Global<v8::Object>,
    layer_name: Option<String>,
    supports_text: Option<String>,
}

#[derive(Default)]
pub(crate) struct CssImportRuleStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, CssImportRuleRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CssImportRuleStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "CSSImportRule", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<CssImportRuleStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "CSSImportRule",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "href", get_href)?;
    crate::webidl::define_accessor(scope, prototype, "media", get_media, set_media)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "styleSheet", get_style_sheet)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "layerName", get_layer_name)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "supportsText", get_supports_text)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::css_rule::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<CssImportRuleStore>()
        .ok_or_else(|| "CSSImportRule state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor");
}

fn take_parenthesized(value: &str, function: &str) -> Option<(String, String)> {
    let trimmed = value.trim_start();
    let prefix = format!("{function}(");
    if !trimmed.to_ascii_lowercase().starts_with(&prefix) {
        return None;
    }
    let mut depth = 0_u32;
    let mut quote = None;
    for (index, character) in trimmed.char_indices() {
        if let Some(current) = quote {
            if character == current {
                quote = None;
            }
        } else if character == '\'' || character == '"' {
            quote = Some(character);
        } else if character == '(' {
            depth += 1;
        } else if character == ')' {
            depth -= 1;
            if depth == 0 {
                let content = trimmed[prefix.len()..index].trim().to_owned();
                let rest = trimmed[index + 1..].trim_start().to_owned();
                return Some((content, rest));
            }
        }
    }
    None
}

fn parse_prelude(
    prelude: &str,
) -> Result<(String, Option<String>, Option<String>, String), String> {
    let mut rest = prelude.trim().trim_end_matches(';').trim().to_owned();
    let href;
    if let Some((url, remaining)) = take_parenthesized(&rest, "url") {
        href = url.trim_matches(['\'', '"']).to_owned();
        rest = remaining;
    } else if rest.starts_with('"') || rest.starts_with('\'') {
        let quote = rest.chars().next().expect("quoted import");
        let end = rest[1..]
            .find(quote)
            .map(|index| index + 1)
            .ok_or_else(|| "Import URL is unterminated".to_owned())?;
        href = rest[1..end].to_owned();
        rest = rest[end + 1..].trim_start().to_owned();
    } else {
        return Err("Import rule requires a URL".to_owned());
    }
    let mut layer_name = None;
    if let Some((layer, remaining)) = take_parenthesized(&rest, "layer") {
        layer_name = Some(layer);
        rest = remaining;
    } else if rest.to_ascii_lowercase().starts_with("layer") {
        layer_name = Some(String::new());
        rest = rest["layer".len()..].trim_start().to_owned();
    }
    let mut supports_text = None;
    if let Some((supports, remaining)) = take_parenthesized(&rest, "supports") {
        supports_text = Some(supports);
        rest = remaining;
    }
    Ok((href, layer_name, supports_text, rest.trim().to_owned()))
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    prelude: &str,
    parent_style_sheet: Option<v8::Local<'_, v8::Object>>,
    parent_rule: Option<v8::Local<'_, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let (href, layer_name, supports_text, media_text) = parse_prelude(prelude)?;
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create CSSImportRule".to_owned());
    }
    let media = super::media_list::create(scope, &media_text)?;
    let style_sheet =
        super::css_style_sheet::create_imported(scope, href.clone(), &media_text, object)?;
    super::css_rule::attach(
        scope,
        object,
        3,
        String::new(),
        parent_style_sheet,
        parent_rule,
    );
    let media = v8::Global::new(scope, media);
    let style_sheet = v8::Global::new(scope, style_sheet);
    scope
        .get_slot_mut::<CssImportRuleStore>()
        .ok_or_else(|| "CSSImportRule state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            CssImportRuleRecord {
                href,
                media,
                style_sheet,
                layer_name,
                supports_text,
            },
        );
    Ok(object)
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<CssImportRuleRecord> {
    scope
        .get_slot::<CssImportRuleStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_href(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this())
        && let Some(href) = v8::String::new(scope, &record.href)
    {
        result.set(href.into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_media(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.media).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_media(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let text = crate::webidl::value_to_string(scope, arguments.get(0));
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    super::media_list::set_text(scope, v8::Local::new(scope, &record.media), &text);
}

fn get_style_sheet(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.style_sheet).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn return_nullable(
    scope: &mut v8::PinScope<'_, '_>,
    value: Option<Option<String>>,
    mut result: v8::ReturnValue<'_>,
) {
    match value {
        Some(Some(value)) => {
            if let Some(value) = v8::String::new(scope, &value) {
                result.set(value.into());
            }
        }
        Some(None) => result.set(v8::null(scope).into()),
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}

fn get_layer_name(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let value = record(scope, arguments.this()).map(|record| record.layer_name);
    return_nullable(scope, value, result);
}

fn get_supports_text(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let value = record(scope, arguments.this()).map(|record| record.supports_text);
    return_nullable(scope, value, result);
}

pub(crate) fn serialize(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<String> {
    let record = record(scope, object)?;
    let mut output = format!("@import url(\"{}\")", record.href);
    if let Some(layer) = record.layer_name {
        if layer.is_empty() {
            output.push_str(" layer");
        } else {
            output.push_str(&format!(" layer({layer})"));
        }
    }
    if let Some(supports) = record.supports_text {
        output.push_str(&format!(" supports({supports})"));
    }
    let media = v8::Local::new(scope, &record.media);
    let text = super::media_list::text(scope, media)?;
    if !text.is_empty() {
        output.push(' ');
        output.push_str(&text);
    }
    output.push(';');
    Some(output)
}
