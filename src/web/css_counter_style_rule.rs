use std::collections::HashMap;

#[derive(Clone)]
struct CssCounterStyleRuleRecord {
    name: String,
    system: String,
    symbols: String,
    additive_symbols: String,
    negative: String,
    prefix: String,
    suffix: String,
    range: String,
    pad: String,
    speak_as: String,
    fallback: String,
}

#[derive(Default)]
pub(crate) struct CssCounterStyleRuleStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, CssCounterStyleRuleRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CssCounterStyleRuleStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "CSSCounterStyleRule", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<CssCounterStyleRuleStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "CSSCounterStyleRule",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_accessor(scope, prototype, "name", get_name, set_name)?;
    crate::webidl::define_accessor(scope, prototype, "system", get_system, set_system)?;
    crate::webidl::define_accessor(scope, prototype, "symbols", get_symbols, set_symbols)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "additiveSymbols",
        get_additive_symbols,
        set_additive_symbols,
    )?;
    crate::webidl::define_accessor(scope, prototype, "negative", get_negative, set_negative)?;
    crate::webidl::define_accessor(scope, prototype, "prefix", get_prefix, set_prefix)?;
    crate::webidl::define_accessor(scope, prototype, "suffix", get_suffix, set_suffix)?;
    crate::webidl::define_accessor(scope, prototype, "range", get_range, set_range)?;
    crate::webidl::define_accessor(scope, prototype, "pad", get_pad, set_pad)?;
    crate::webidl::define_accessor(scope, prototype, "speakAs", get_speak_as, set_speak_as)?;
    crate::webidl::define_accessor(scope, prototype, "fallback", get_fallback, set_fallback)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::css_rule::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<CssCounterStyleRuleStore>()
        .ok_or_else(|| "CSSCounterStyleRule state was not prepared".to_owned())?
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

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    name: String,
    body: &str,
    parent_style_sheet: Option<v8::Local<'_, v8::Object>>,
    parent_rule: Option<v8::Local<'_, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create CSSCounterStyleRule".to_owned());
    }
    let declarations = super::css_style_declaration::create(scope, body, Some(object), None)?;
    let value = |name: &str| {
        super::css_style_declaration::named_value(scope, declarations, name).unwrap_or_default()
    };
    let record = CssCounterStyleRuleRecord {
        name,
        system: value("system"),
        symbols: value("symbols"),
        additive_symbols: value("additive-symbols"),
        negative: value("negative"),
        prefix: value("prefix"),
        suffix: value("suffix"),
        range: value("range"),
        pad: value("pad"),
        speak_as: value("speak-as"),
        fallback: value("fallback"),
    };
    super::css_rule::attach(
        scope,
        object,
        11,
        String::new(),
        parent_style_sheet,
        parent_rule,
    );
    scope
        .get_slot_mut::<CssCounterStyleRuleStore>()
        .ok_or_else(|| "CSSCounterStyleRule state was not prepared".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), record);
    Ok(object)
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<CssCounterStyleRuleRecord> {
    scope
        .get_slot::<CssCounterStyleRuleStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn return_value(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    select: fn(&CssCounterStyleRuleRecord) -> &str,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, object)
        && let Some(value) = v8::String::new(scope, select(&record))
    {
        result.set(value.into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn update_value(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    update: fn(&mut CssCounterStyleRuleRecord, String),
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    if let Some(record) = scope
        .get_slot_mut::<CssCounterStyleRuleStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        update(record, value);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_name(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_value(s, a.this(), |v| &v.name, r);
}
fn set_name(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    update_value(s, a, |v, x| v.name = x);
}
fn get_system(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_value(s, a.this(), |v| &v.system, r);
}
fn set_system(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    update_value(s, a, |v, x| v.system = x);
}
fn get_symbols(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_value(s, a.this(), |v| &v.symbols, r);
}
fn set_symbols(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    update_value(s, a, |v, x| v.symbols = x);
}
fn get_additive_symbols(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_value(s, a.this(), |v| &v.additive_symbols, r);
}
fn set_additive_symbols(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    update_value(s, a, |v, x| v.additive_symbols = x);
}
fn get_negative(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_value(s, a.this(), |v| &v.negative, r);
}
fn set_negative(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    update_value(s, a, |v, x| v.negative = x);
}
fn get_prefix(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_value(s, a.this(), |v| &v.prefix, r);
}
fn set_prefix(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    update_value(s, a, |v, x| v.prefix = x);
}
fn get_suffix(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_value(s, a.this(), |v| &v.suffix, r);
}
fn set_suffix(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    update_value(s, a, |v, x| v.suffix = x);
}
fn get_range(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_value(s, a.this(), |v| &v.range, r);
}
fn set_range(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    update_value(s, a, |v, x| v.range = x);
}
fn get_pad(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_value(s, a.this(), |v| &v.pad, r);
}
fn set_pad(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    update_value(s, a, |v, x| v.pad = x);
}
fn get_speak_as(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_value(s, a.this(), |v| &v.speak_as, r);
}
fn set_speak_as(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    update_value(s, a, |v, x| v.speak_as = x);
}
fn get_fallback(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_value(s, a.this(), |v| &v.fallback, r);
}
fn set_fallback(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    update_value(s, a, |v, x| v.fallback = x);
}

fn push_descriptor(output: &mut String, name: &str, value: &str) {
    if !value.is_empty() {
        output.push_str(name);
        output.push_str(": ");
        output.push_str(value);
        output.push_str("; ");
    }
}

pub(crate) fn serialize(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<String> {
    let record = record(scope, object)?;
    let mut declarations = String::new();
    push_descriptor(&mut declarations, "system", &record.system);
    push_descriptor(&mut declarations, "symbols", &record.symbols);
    push_descriptor(
        &mut declarations,
        "additive-symbols",
        &record.additive_symbols,
    );
    push_descriptor(&mut declarations, "negative", &record.negative);
    push_descriptor(&mut declarations, "prefix", &record.prefix);
    push_descriptor(&mut declarations, "suffix", &record.suffix);
    push_descriptor(&mut declarations, "range", &record.range);
    push_descriptor(&mut declarations, "pad", &record.pad);
    push_descriptor(&mut declarations, "speak-as", &record.speak_as);
    push_descriptor(&mut declarations, "fallback", &record.fallback);
    Some(format!(
        "@counter-style {} {{ {} }}",
        record.name,
        declarations.trim_end()
    ))
}
