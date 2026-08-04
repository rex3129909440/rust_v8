use std::collections::HashMap;

#[derive(Clone)]
struct CssLayerStatementRuleRecord {
    names: Vec<String>,
    name_list: v8::Global<v8::Array>,
}

#[derive(Default)]
pub(crate) struct CssLayerStatementRuleStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, CssLayerStatementRuleRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CssLayerStatementRuleStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "CSSLayerStatementRule", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<CssLayerStatementRuleStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "CSSLayerStatementRule",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "nameList", get_name_list)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::css_rule::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<CssLayerStatementRuleStore>()
        .ok_or_else(|| "CSSLayerStatementRule state was not prepared".to_owned())?
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
    names: Vec<String>,
    parent_style_sheet: Option<v8::Local<'_, v8::Object>>,
    parent_rule: Option<v8::Local<'_, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    if names.is_empty() {
        return Err("Layer statement requires a name".to_owned());
    }
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create CSSLayerStatementRule".to_owned());
    }
    let list = v8::Array::new(scope, names.len() as i32);
    for (index, name) in names.iter().enumerate() {
        let value = crate::webidl::string(scope, name)?;
        let _ = list.set_index(scope, index as u32, value.into());
    }
    super::css_rule::attach(
        scope,
        object,
        0,
        String::new(),
        parent_style_sheet,
        parent_rule,
    );
    let name_list = v8::Global::new(scope, list);
    scope
        .get_slot_mut::<CssLayerStatementRuleStore>()
        .ok_or_else(|| "CSSLayerStatementRule state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            CssLayerStatementRuleRecord { names, name_list },
        );
    Ok(object)
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<CssLayerStatementRuleRecord> {
    scope
        .get_slot::<CssLayerStatementRuleStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_name_list(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.name_list).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn serialize(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<String> {
    let record = record(scope, object)?;
    Some(format!("@layer {};", record.names.join(", ")))
}
