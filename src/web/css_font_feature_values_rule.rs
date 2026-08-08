use std::collections::{BTreeMap, HashMap};

#[derive(Clone)]
struct CssFontFeatureValuesRuleRecord {
    font_family: String,
    annotation: v8::Global<v8::Object>,
    ornaments: v8::Global<v8::Object>,
    stylistic: v8::Global<v8::Object>,
    swash: v8::Global<v8::Object>,
    character_variant: v8::Global<v8::Object>,
    styleset: v8::Global<v8::Object>,
}

#[derive(Default)]
pub(crate) struct CssFontFeatureValuesRuleStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, CssFontFeatureValuesRuleRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CssFontFeatureValuesRuleStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "CSSFontFeatureValuesRule", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<CssFontFeatureValuesRuleStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "CSSFontFeatureValuesRule",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "fontFamily",
        get_font_family,
        set_font_family,
    )?;
    crate::webidl::define_readonly_accessor(scope, prototype, "annotation", get_annotation)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "ornaments", get_ornaments)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "stylistic", get_stylistic)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "swash", get_swash)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "characterVariant",
        get_character_variant,
    )?;
    crate::webidl::define_readonly_accessor(scope, prototype, "styleset", get_styleset)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::css_rule::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<CssFontFeatureValuesRuleStore>()
        .ok_or_else(|| "CSSFontFeatureValuesRule state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    font_family: String,
    body: &str,
    parent_style_sheet: Option<v8::Local<'_, v8::Object>>,
    parent_rule: Option<v8::Local<'_, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let rule = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, rule, prototype.into()) != Some(true) {
        return Err("cannot create CSSFontFeatureValuesRule".to_owned());
    }
    let parsed = parse_feature_blocks(body);
    let annotation = super::css_font_feature_values_map::create(
        scope,
        parsed.get("annotation").cloned().unwrap_or_default(),
    )?;
    let ornaments = super::css_font_feature_values_map::create(
        scope,
        parsed.get("ornaments").cloned().unwrap_or_default(),
    )?;
    let stylistic = super::css_font_feature_values_map::create(
        scope,
        parsed.get("stylistic").cloned().unwrap_or_default(),
    )?;
    let swash = super::css_font_feature_values_map::create(
        scope,
        parsed.get("swash").cloned().unwrap_or_default(),
    )?;
    let character_variant = super::css_font_feature_values_map::create(
        scope,
        parsed.get("character-variant").cloned().unwrap_or_default(),
    )?;
    let styleset = super::css_font_feature_values_map::create(
        scope,
        parsed.get("styleset").cloned().unwrap_or_default(),
    )?;
    let record = CssFontFeatureValuesRuleRecord {
        font_family,
        annotation: v8::Global::new(scope, annotation),
        ornaments: v8::Global::new(scope, ornaments),
        stylistic: v8::Global::new(scope, stylistic),
        swash: v8::Global::new(scope, swash),
        character_variant: v8::Global::new(scope, character_variant),
        styleset: v8::Global::new(scope, styleset),
    };
    scope
        .get_slot_mut::<CssFontFeatureValuesRuleStore>()
        .ok_or_else(|| "CSSFontFeatureValuesRule state was not prepared".to_owned())?
        .records
        .insert(rule.get_identity_hash().get(), record);
    let css_text = serialize(scope, rule).unwrap_or_default();
    super::css_rule::attach(scope, rule, 14, css_text, parent_style_sheet, parent_rule);
    Ok(rule)
}

pub(crate) fn serialize(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<String> {
    let record = record(scope, object)?;
    let mut blocks = String::new();
    append_block(scope, &mut blocks, "annotation", &record.annotation);
    append_block(scope, &mut blocks, "ornaments", &record.ornaments);
    append_block(scope, &mut blocks, "stylistic", &record.stylistic);
    append_block(scope, &mut blocks, "swash", &record.swash);
    append_block(
        scope,
        &mut blocks,
        "character-variant",
        &record.character_variant,
    );
    append_block(scope, &mut blocks, "styleset", &record.styleset);
    Some(format!(
        "@font-feature-values {} {{ {} }}",
        record.font_family,
        blocks.trim()
    ))
}

fn append_block(
    scope: &v8::PinScope<'_, '_>,
    output: &mut String,
    name: &str,
    map: &v8::Global<v8::Object>,
) {
    let map = v8::Local::new(scope, map);
    let Some(entries) = super::css_font_feature_values_map::snapshot(scope, map) else {
        return;
    };
    if entries.is_empty() {
        return;
    }
    output.push('@');
    output.push_str(name);
    output.push_str(" { ");
    for (key, values) in entries {
        output.push_str(&key);
        output.push_str(":");
        for value in values {
            output.push(' ');
            output.push_str(&value.to_string());
        }
        output.push_str("; ");
    }
    output.push_str("} ");
}

fn parse_feature_blocks(body: &str) -> HashMap<String, BTreeMap<String, Vec<u32>>> {
    let mut output = HashMap::new();
    let mut rest = body;
    while let Some(at) = rest.find('@') {
        rest = &rest[at + 1..];
        let Some(open) = rest.find('{') else {
            break;
        };
        let category = rest[..open].trim().to_ascii_lowercase();
        let after_open = &rest[open + 1..];
        let Some(close) = after_open.find('}') else {
            break;
        };
        let declarations = &after_open[..close];
        let mut values = BTreeMap::new();
        for declaration in declarations.split(';') {
            let Some((name, numbers)) = declaration.split_once(':') else {
                continue;
            };
            let numbers = numbers
                .split_ascii_whitespace()
                .filter_map(|value| value.parse::<u32>().ok())
                .collect::<Vec<_>>();
            if !name.trim().is_empty() && !numbers.is_empty() {
                values.insert(name.trim().to_owned(), numbers);
            }
        }
        output.insert(category, values);
        rest = &after_open[close + 1..];
    }
    output
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<CssFontFeatureValuesRuleRecord> {
    scope
        .get_slot::<CssFontFeatureValuesRuleStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'CSSFontFeatureValuesRule': Illegal constructor",
    );
}

fn get_font_family(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(value) = v8::String::new(scope, &record.font_family) {
        result.set(value.into());
    }
}

fn set_font_family(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let family = crate::webidl::value_to_string(scope, arguments.get(0));
    let Some(record) = scope
        .get_slot_mut::<CssFontFeatureValuesRuleStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    record.font_family = family;
}

fn return_map(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&CssFontFeatureValuesRuleRecord) -> &v8::Global<v8::Object>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    result.set(v8::Local::new(scope, select(&record)).into());
}

fn get_annotation(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_map(s, a, r, |x| &x.annotation)
}
fn get_ornaments(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_map(s, a, r, |x| &x.ornaments)
}
fn get_stylistic(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_map(s, a, r, |x| &x.stylistic)
}
fn get_swash(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_map(s, a, r, |x| &x.swash)
}
fn get_character_variant(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_map(s, a, r, |x| &x.character_variant)
}
fn get_styleset(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_map(s, a, r, |x| &x.styleset)
}
