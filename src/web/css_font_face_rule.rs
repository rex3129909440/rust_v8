use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct CssFontFaceRuleStore {
    constructor: crate::webidl::RealmConstructor,
    styles: HashMap<i32, v8::Global<v8::Object>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CssFontFaceRuleStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "CSSFontFaceRule", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<CssFontFaceRuleStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "CSSFontFaceRule",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "style", get_style)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::css_rule::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<CssFontFaceRuleStore>()
        .ok_or_else(|| "CSSFontFaceRule state was not prepared".to_owned())?
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
    body: &str,
    parent_style_sheet: Option<v8::Local<'_, v8::Object>>,
    parent_rule: Option<v8::Local<'_, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create CSSFontFaceRule".to_owned());
    }
    let style = super::css_style_declaration::create(scope, body, Some(object), None)?;
    if let Some(source) = super::css_style_declaration::named_value(scope, style, "src")
        && let Some(normalized) = normalize_url(&source)
    {
        super::css_style_declaration::set_named_value(scope, style, "src", normalized);
    }
    super::css_rule::attach(
        scope,
        object,
        5,
        String::new(),
        parent_style_sheet,
        parent_rule,
    );
    let style = v8::Global::new(scope, style);
    scope
        .get_slot_mut::<CssFontFaceRuleStore>()
        .ok_or_else(|| "CSSFontFaceRule state was not prepared".to_owned())?
        .styles
        .insert(object.get_identity_hash().get(), style);
    Ok(object)
}

fn normalize_url(value: &str) -> Option<String> {
    let value = value.trim();
    let lower = value.to_ascii_lowercase();
    if !lower.starts_with("url(") || !value.ends_with(')') {
        return None;
    }
    let inner = value[4..value.len() - 1].trim();
    if inner.starts_with('"') || inner.starts_with('\'') {
        None
    } else {
        Some(format!("url(\"{inner}\")"))
    }
}

fn style(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<v8::Global<v8::Object>> {
    scope
        .get_slot::<CssFontFaceRuleStore>()?
        .styles
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_style(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(style) = style(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &style).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn serialize(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<String> {
    let style = style(scope, object)?;
    let declarations =
        super::css_style_declaration::serialize(scope, v8::Local::new(scope, &style))?;
    Some(format!("@font-face {{ {declarations} }}"))
}
