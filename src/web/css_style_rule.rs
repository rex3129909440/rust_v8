use std::collections::HashMap;

#[derive(Clone)]
struct CssStyleRuleRecord {
    selector: String,
    style: v8::Global<v8::Object>,
    style_map: v8::Global<v8::Object>,
}

#[derive(Default)]
pub(crate) struct CssStyleRuleStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, CssStyleRuleRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CssStyleRuleStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "CSSStyleRule", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<CssStyleRuleStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "CSSStyleRule",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "selectorText",
        get_selector_text,
        set_selector_text,
    )?;
    crate::webidl::define_accessor(scope, prototype, "style", get_style, set_style)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "styleMap", get_style_map)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "cssRules", get_css_rules)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "deleteRule",
        1,
        super::css_grouping_rule::delete_rule,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "insertRule",
        1,
        super::css_grouping_rule::insert_rule,
    )?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::css_rule::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<CssStyleRuleStore>()
        .ok_or_else(|| "CSSStyleRule state was not prepared".to_owned())?
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
    selector: String,
    declarations: String,
    parent_style_sheet: Option<v8::Local<'_, v8::Object>>,
    parent_rule: Option<v8::Local<'_, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create CSSStyleRule".to_owned());
    }
    let (declaration_text, nested_text) = split_body(&declarations);
    let style_map = super::style_property_map::create(scope)?;
    let style = super::css_style_declaration::create(
        scope,
        &declaration_text,
        Some(object),
        Some(style_map),
    )?;
    super::css_grouping_rule::attach(scope, object, Vec::new())?;
    super::css_rule::attach(
        scope,
        object,
        1,
        String::new(),
        parent_style_sheet,
        parent_rule,
    );
    let record = CssStyleRuleRecord {
        selector,
        style: v8::Global::new(scope, style),
        style_map: v8::Global::new(scope, style_map),
    };
    scope
        .get_slot_mut::<CssStyleRuleStore>()
        .ok_or_else(|| "CSSStyleRule state was not prepared".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), record);
    if !nested_text.is_empty() {
        let nested = super::css_style_sheet::parse_rules(
            scope,
            &nested_text,
            parent_style_sheet,
            Some(object),
        )?;
        super::css_grouping_rule::replace_rules(scope, object, nested);
    }
    Ok(object)
}

fn split_body(body: &str) -> (String, String) {
    let Some(open) = body.find('{') else {
        return (body.to_owned(), String::new());
    };
    let boundary = body[..open].rfind(';').map(|index| index + 1).unwrap_or(0);
    (
        body[..boundary].trim().to_owned(),
        body[boundary..].trim().to_owned(),
    )
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<CssStyleRuleRecord> {
    scope
        .get_slot::<CssStyleRuleStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn selector_and_properties(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<(String, Vec<super::css_style_declaration::CssProperty>)> {
    let record = record(scope, object)?;
    let style = v8::Local::new(scope, &record.style);
    Some((
        record.selector,
        super::css_style_declaration::properties(scope, style)?,
    ))
}

fn get_selector_text(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(selector) = v8::String::new(scope, &record.selector) {
            result.set(selector.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_selector_text(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let selector = crate::webidl::value_to_string(scope, arguments.get(0))
        .trim()
        .to_owned();
    if selector.is_empty() {
        return;
    }
    if let Some(record) = scope.get_slot_mut::<CssStyleRuleStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) {
        record.selector = selector;
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
    let style = v8::Local::new(scope, &record.style);
    let text = crate::webidl::value_to_string(scope, arguments.get(0));
    super::css_style_declaration::set_text(scope, style, &text);
}

fn get_style_map(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.style_map).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_css_rules(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(list) = super::css_grouping_rule::list(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &list).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn serialize(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<String> {
    let record = record(scope, object)?;
    let style = v8::Local::new(scope, &record.style);
    let declarations = super::css_style_declaration::serialize(scope, style)?;
    Some(if declarations.is_empty() {
        format!("{} {{}}", record.selector)
    } else {
        format!("{} {{ {} }}", record.selector, declarations)
    })
}
