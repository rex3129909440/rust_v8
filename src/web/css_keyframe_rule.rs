use std::collections::HashMap;

#[derive(Clone)]
struct CssKeyframeRuleRecord {
    key_text: String,
    style: v8::Global<v8::Object>,
}

#[derive(Default)]
pub(crate) struct CssKeyframeRuleStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, CssKeyframeRuleRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CssKeyframeRuleStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "CSSKeyframeRule", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<CssKeyframeRuleStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "CSSKeyframeRule",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_accessor(scope, prototype, "keyText", get_key_text, set_key_text)?;
    crate::webidl::define_accessor(scope, prototype, "style", get_style, set_style)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::css_rule::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<CssKeyframeRuleStore>()
        .ok_or_else(|| "CSSKeyframeRule state was not prepared".to_owned())?
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

fn normalize_key(value: &str) -> Option<String> {
    let value = value.trim();
    if value.eq_ignore_ascii_case("from") {
        Some("0%".to_owned())
    } else if value.eq_ignore_ascii_case("to") {
        Some("100%".to_owned())
    } else if let Some(number) = value.strip_suffix('%') {
        let number = number.trim().parse::<f64>().ok()?;
        if (0.0..=100.0).contains(&number) {
            Some(format!("{number}%"))
        } else {
            None
        }
    } else {
        None
    }
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    key_text: &str,
    declarations: &str,
    parent_style_sheet: Option<v8::Local<'_, v8::Object>>,
    parent_rule: Option<v8::Local<'_, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let key_text = normalize_key(key_text).ok_or_else(|| "Invalid keyframe selector".to_owned())?;
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create CSSKeyframeRule".to_owned());
    }
    let style = super::css_style_declaration::create(scope, declarations, Some(object), None)?;
    crate::webidl::define_accessor(scope, style, "opacity", get_opacity, set_opacity)?;
    normalize_opacity(scope, style);
    super::css_rule::attach(
        scope,
        object,
        8,
        String::new(),
        parent_style_sheet,
        parent_rule,
    );
    let style = v8::Global::new(scope, style);
    scope
        .get_slot_mut::<CssKeyframeRuleStore>()
        .ok_or_else(|| "CSSKeyframeRule state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            CssKeyframeRuleRecord { key_text, style },
        );
    Ok(object)
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<CssKeyframeRuleRecord> {
    scope
        .get_slot::<CssKeyframeRuleStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn key_text(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<String> {
    Some(record(scope, object)?.key_text)
}

fn get_key_text(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(key) = key_text(scope, arguments.this())
        && let Some(key) = v8::String::new(scope, &key)
    {
        result.set(key.into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_key_text(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    let Some(value) = normalize_key(&value) else {
        return;
    };
    if let Some(record) = scope
        .get_slot_mut::<CssKeyframeRuleStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.key_text = value;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_style(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.style).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_style(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let text = crate::webidl::value_to_string(scope, arguments.get(0));
    let style = v8::Local::new(scope, &record.style);
    super::css_style_declaration::set_text(scope, style, &text);
    normalize_opacity(scope, style);
}

fn normalize_opacity(scope: &mut v8::PinScope<'_, '_>, style: v8::Local<'_, v8::Object>) {
    if let Some(mut value) = super::css_style_declaration::named_value(scope, style, "opacity")
        && value.starts_with('.')
    {
        value.insert(0, '0');
        super::css_style_declaration::set_named_value(scope, style, "opacity", value);
    }
}

fn get_opacity(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(value) =
        super::css_style_declaration::named_value(scope, arguments.this(), "opacity")
        && let Some(value) = v8::String::new(scope, &value)
    {
        result.set(value.into());
    }
}

fn set_opacity(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let mut value = crate::webidl::value_to_string(scope, arguments.get(0));
    if value.starts_with('.') {
        value.insert(0, '0');
    }
    super::css_style_declaration::set_named_value(scope, arguments.this(), "opacity", value);
}

pub(crate) fn serialize(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<String> {
    let record = record(scope, object)?;
    let declarations =
        super::css_style_declaration::serialize(scope, v8::Local::new(scope, &record.style))?;
    Some(format!("{} {{ {} }}", record.key_text, declarations))
}
