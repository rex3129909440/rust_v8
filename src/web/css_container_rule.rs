use std::collections::HashMap;

#[derive(Clone)]
struct CssContainerRuleRecord {
    name: String,
    query: String,
    conditions: v8::Global<v8::Array>,
}

#[derive(Default)]
pub(crate) struct CssContainerRuleStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, CssContainerRuleRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CssContainerRuleStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "CSSContainerRule", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<CssContainerRuleStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "CSSContainerRule",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "containerName", get_container_name)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "containerQuery",
        get_container_query,
    )?;
    crate::webidl::define_readonly_accessor(scope, prototype, "conditions", get_conditions)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::css_condition_rule::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<CssContainerRuleStore>()
        .ok_or_else(|| "CSSContainerRule state was not prepared".to_owned())?
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

fn split_condition(condition: &str) -> (String, String) {
    let condition = condition.trim();
    if condition.starts_with('(') || condition.to_ascii_lowercase().starts_with("style(") {
        return (String::new(), condition.to_owned());
    }
    if let Some(index) = condition.find(char::is_whitespace) {
        (
            condition[..index].to_owned(),
            condition[index..].trim().to_owned(),
        )
    } else {
        (condition.to_owned(), String::new())
    }
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    condition: String,
    body: &str,
    parent_style_sheet: Option<v8::Local<'_, v8::Object>>,
    parent_rule: Option<v8::Local<'_, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create CSSContainerRule".to_owned());
    }
    let (name, query) = split_condition(&condition);
    let condition_object = v8::Object::new(scope);
    let name_key = crate::webidl::string(scope, "name")?;
    let name_value = crate::webidl::string(scope, &name)?;
    let _ = condition_object.set(scope, name_key.into(), name_value.into());
    let query_key = crate::webidl::string(scope, "query")?;
    let query_value = crate::webidl::string(scope, &query)?;
    let _ = condition_object.set(scope, query_key.into(), query_value.into());
    let conditions = v8::Array::new(scope, 1);
    let _ = conditions.set_index(scope, 0, condition_object.into());
    super::css_grouping_rule::attach(scope, object, Vec::new())?;
    super::css_condition_rule::attach(scope, object, condition);
    super::css_rule::attach(
        scope,
        object,
        0,
        String::new(),
        parent_style_sheet,
        parent_rule,
    );
    let conditions = v8::Global::new(scope, conditions);
    scope
        .get_slot_mut::<CssContainerRuleStore>()
        .ok_or_else(|| "CSSContainerRule state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            CssContainerRuleRecord {
                name,
                query,
                conditions,
            },
        );
    let nested =
        super::css_style_sheet::parse_rules(scope, body, parent_style_sheet, Some(object))?;
    super::css_grouping_rule::replace_rules(scope, object, nested);
    Ok(object)
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<CssContainerRuleRecord> {
    scope
        .get_slot::<CssContainerRuleStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn return_text(
    scope: &mut v8::PinScope<'_, '_>,
    text: Option<String>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(text) = text
        && let Some(text) = v8::String::new(scope, &text)
    {
        result.set(text.into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_container_name(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let value = record(scope, arguments.this()).map(|record| record.name);
    return_text(scope, value, result);
}

fn get_container_query(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let value = record(scope, arguments.this()).map(|record| record.query);
    return_text(scope, value, result);
}

fn get_conditions(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.conditions).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn serialize(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<String> {
    let _record = record(scope, object)?;
    let condition = super::css_condition_rule::condition(scope, object)?;
    let body = super::css_grouping_rule::serialized_body(scope, object)?;
    Some(format!("@container {condition} {{\n{body}}}"))
}
