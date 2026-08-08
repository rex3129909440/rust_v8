use std::collections::HashMap;

#[derive(Clone)]
pub(crate) struct CssRuleRecord {
    pub rule_type: u32,
    pub css_text: String,
    pub parent_rule: Option<v8::Global<v8::Object>>,
    pub parent_style_sheet: Option<v8::Global<v8::Object>>,
}

#[derive(Default)]
pub(crate) struct CssRuleStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, CssRuleRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CssRuleStore::default());
}

#[allow(dead_code)]
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "CSSRule", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<CssRuleStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "CSSRule",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "type", get_type)?;
    crate::webidl::define_accessor(scope, prototype, "cssText", get_css_text, set_css_text)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "parentRule", get_parent_rule)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "parentStyleSheet",
        get_parent_style_sheet,
    )?;
    define_constants(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    define_constants(scope, constructor.into())?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<CssRuleStore>()
        .ok_or_else(|| "CSSRule state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn define_constants(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_constant(scope, object, "STYLE_RULE", 1)?;
    crate::webidl::define_constant(scope, object, "CHARSET_RULE", 2)?;
    crate::webidl::define_constant(scope, object, "IMPORT_RULE", 3)?;
    crate::webidl::define_constant(scope, object, "MEDIA_RULE", 4)?;
    crate::webidl::define_constant(scope, object, "FONT_FACE_RULE", 5)?;
    crate::webidl::define_constant(scope, object, "PAGE_RULE", 6)?;
    crate::webidl::define_constant(scope, object, "MARGIN_RULE", 9)?;
    crate::webidl::define_constant(scope, object, "NAMESPACE_RULE", 10)?;
    crate::webidl::define_constant(scope, object, "KEYFRAMES_RULE", 7)?;
    crate::webidl::define_constant(scope, object, "KEYFRAME_RULE", 8)?;
    crate::webidl::define_constant(scope, object, "COUNTER_STYLE_RULE", 11)?;
    crate::webidl::define_constant(scope, object, "FONT_FEATURE_VALUES_RULE", 14)?;
    crate::webidl::define_constant(scope, object, "SUPPORTS_RULE", 12)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor");
}

pub(crate) fn attach(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    rule_type: u32,
    css_text: String,
    parent_style_sheet: Option<v8::Local<'_, v8::Object>>,
    parent_rule: Option<v8::Local<'_, v8::Object>>,
) {
    let record = CssRuleRecord {
        rule_type,
        css_text,
        parent_rule: parent_rule.map(|rule| v8::Global::new(scope, rule)),
        parent_style_sheet: parent_style_sheet.map(|sheet| v8::Global::new(scope, sheet)),
    };
    scope
        .get_slot_mut::<CssRuleStore>()
        .expect("CSSRule state")
        .records
        .insert(object.get_identity_hash().get(), record);
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<CssRuleRecord> {
    scope
        .get_slot::<CssRuleStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn serialized(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<String> {
    super::css_function_rule::serialize(scope, object)
        .or_else(|| super::css_function_declarations::serialize(scope, object))
        .or_else(|| super::css_font_feature_values_rule::serialize(scope, object))
        .or_else(|| super::css_style_rule::serialize(scope, object))
        .or_else(|| super::css_keyframes_rule::serialize(scope, object))
        .or_else(|| super::css_keyframe_rule::serialize(scope, object))
        .or_else(|| super::css_import_rule::serialize(scope, object))
        .or_else(|| super::css_font_palette_values_rule::serialize(scope, object))
        .or_else(|| super::css_font_face_rule::serialize(scope, object))
        .or_else(|| super::css_counter_style_rule::serialize(scope, object))
        .or_else(|| super::css_container_rule::serialize(scope, object))
        .or_else(|| super::css_supports_rule::serialize(scope, object))
        .or_else(|| super::css_starting_style_rule::serialize(scope, object))
        .or_else(|| super::css_scope_rule::serialize(scope, object))
        .or_else(|| super::css_property_rule::serialize(scope, object))
        .or_else(|| super::css_position_try_rule::serialize(scope, object))
        .or_else(|| super::css_page_rule::serialize(scope, object))
        .or_else(|| super::css_nested_declarations::serialize(scope, object))
        .or_else(|| super::css_namespace_rule::serialize(scope, object))
        .or_else(|| super::css_media_rule::serialize(scope, object))
        .or_else(|| super::css_margin_rule::serialize(scope, object))
        .or_else(|| super::css_layer_statement_rule::serialize(scope, object))
        .or_else(|| super::css_layer_block_rule::serialize(scope, object))
        .or_else(|| record(scope, object).map(|record| record.css_text))
}

fn get_type(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new_from_unsigned(scope, record.rule_type).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_css_text(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(text) = serialized(scope, arguments.this()) {
        if let Some(text) = v8::String::new(scope, &text) {
            result.set(text.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_css_text(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let text = crate::webidl::value_to_string(scope, arguments.get(0));
    if let Some(record) = scope.get_slot_mut::<CssRuleStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) {
        record.css_text = text;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_parent_rule(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match record(scope, arguments.this()) {
        Some(record) => match record.parent_rule {
            Some(parent) => result.set(v8::Local::new(scope, &parent).into()),
            None => result.set(v8::null(scope).into()),
        },
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}

fn get_parent_style_sheet(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match record(scope, arguments.this()) {
        Some(record) => match record.parent_style_sheet {
            Some(parent) => result.set(v8::Local::new(scope, &parent).into()),
            None => result.set(v8::null(scope).into()),
        },
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}
