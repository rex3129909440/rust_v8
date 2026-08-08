use std::collections::HashMap;

#[derive(Clone)]
struct CssPageRuleRecord {
    selector: String,
    style: v8::Global<v8::Object>,
}

#[derive(Default)]
pub(crate) struct CssPageRuleStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, CssPageRuleRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CssPageRuleStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "CSSPageRule", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<CssPageRuleStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "CSSPageRule",
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
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::css_grouping_rule::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<CssPageRuleStore>()
        .ok_or_else(|| "CSSPageRule state was not prepared".to_owned())?
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
    body: &str,
    parent_style_sheet: Option<v8::Local<'_, v8::Object>>,
    parent_rule: Option<v8::Local<'_, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create CSSPageRule".to_owned());
    }
    let (declaration_text, nested_text) = split_body(body);
    let style = super::css_style_declaration::create(scope, &declaration_text, Some(object), None)?;
    crate::webidl::define_accessor(scope, style, "margin", get_margin, set_margin)?;
    crate::webidl::define_accessor(scope, style, "size", get_size, set_size)?;
    if let Some(size) = super::css_style_declaration::named_value(scope, style, "size")
        && !size.is_empty()
    {
        super::css_style_declaration::set_named_value(
            scope,
            style,
            "size",
            size.to_ascii_lowercase(),
        );
    }
    super::css_grouping_rule::attach(scope, object, Vec::new())?;
    super::css_rule::attach(
        scope,
        object,
        6,
        String::new(),
        parent_style_sheet,
        parent_rule,
    );
    let style = v8::Global::new(scope, style);
    scope
        .get_slot_mut::<CssPageRuleStore>()
        .ok_or_else(|| "CSSPageRule state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            CssPageRuleRecord { selector, style },
        );
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
) -> Option<CssPageRuleRecord> {
    scope
        .get_slot::<CssPageRuleStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
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
    let selector = crate::webidl::value_to_string(scope, arguments.get(0));
    if let Some(record) = scope.get_slot_mut::<CssPageRuleStore>().and_then(|store| {
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
    let text = crate::webidl::value_to_string(scope, arguments.get(0));
    super::css_style_declaration::set_text(scope, v8::Local::new(scope, &record.style), &text);
}

fn get_margin(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(value) =
        super::css_style_declaration::named_value(scope, arguments.this(), "margin")
        && let Some(value) = v8::String::new(scope, &value)
    {
        result.set(value.into());
    }
}

fn set_margin(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    super::css_style_declaration::set_named_value(scope, arguments.this(), "margin", value);
}

fn get_size(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(value) = super::css_style_declaration::named_value(scope, arguments.this(), "size")
        && let Some(value) = v8::String::new(scope, &value)
    {
        result.set(value.into());
    }
}

fn set_size(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0)).to_ascii_lowercase();
    super::css_style_declaration::set_named_value(scope, arguments.this(), "size", value);
}

pub(crate) fn serialize(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<String> {
    let record = record(scope, object)?;
    let style = v8::Local::new(scope, &record.style);
    let declarations = super::css_style_declaration::serialize(scope, style)?;
    let selector = if record.selector.trim().is_empty() {
        String::new()
    } else {
        format!(" {}", record.selector.trim())
    };
    let mut body = declarations;
    if let Some(list) = super::css_grouping_rule::list(scope, object)
        && let Some(rules) = super::css_rule_list::rules(scope, v8::Local::new(scope, &list))
    {
        for rule in rules {
            if let Some(text) = super::css_rule::serialized(scope, v8::Local::new(scope, &rule)) {
                if !body.is_empty() {
                    body.push(' ');
                }
                body.push_str(&text);
            }
        }
    }
    Some(format!("@page{} {{ {} }}", selector, body))
}
