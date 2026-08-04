use std::collections::HashMap;

#[derive(Clone, Default)]
struct CssRuleListRecord {
    rules: Vec<v8::Global<v8::Object>>,
}

#[derive(Default)]
pub(crate) struct CssRuleListStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, CssRuleListRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CssRuleListStore::default());
}

#[allow(dead_code)]
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "CSSRuleList", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<CssRuleListStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "CSSRuleList",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "length", get_length)?;
    crate::webidl::define_method(scope, prototype, "item", 1, item)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let iterator = crate::webidl::create_function(
        scope,
        "values",
        0,
        v8::ConstructorBehavior::Throw,
        iterator,
    )?;
    let iterator_key = v8::Symbol::get_iterator(scope);
    if prototype.define_own_property(
        scope,
        iterator_key.into(),
        iterator.into(),
        v8::PropertyAttribute::DONT_ENUM,
    ) != Some(true)
    {
        return Err("cannot define CSSRuleList iterator".to_owned());
    }
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<CssRuleListStore>()
        .ok_or_else(|| "CSSRuleList state was not prepared".to_owned())?
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
    rules: Vec<v8::Local<'_, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create CSSRuleList".to_owned());
    }
    let rules = rules
        .into_iter()
        .map(|rule| v8::Global::new(scope, rule))
        .collect();
    scope
        .get_slot_mut::<CssRuleListStore>()
        .ok_or_else(|| "CSSRuleList state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            CssRuleListRecord { rules },
        );
    refresh(scope, object, 0);
    Ok(object)
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<CssRuleListRecord> {
    scope
        .get_slot::<CssRuleListStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn refresh(scope: &mut v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>, old_length: usize) {
    for index in 0..old_length {
        let _ = object.delete_index(scope, index as u32);
    }
    if let Some(record) = record(scope, object) {
        for (index, rule) in record.rules.iter().enumerate() {
            let rule = v8::Local::new(scope, rule);
            let _ = object.set_index(scope, index as u32, rule.into());
        }
    }
}

pub(crate) fn rules(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<Vec<v8::Global<v8::Object>>> {
    Some(record(scope, object)?.rules)
}

pub(crate) fn replace(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    rules: Vec<v8::Local<'_, v8::Object>>,
) -> bool {
    let Some(old_length) = record(scope, object).map(|record| record.rules.len()) else {
        return false;
    };
    let rules = rules
        .into_iter()
        .map(|rule| v8::Global::new(scope, rule))
        .collect();
    if let Some(record) = scope
        .get_slot_mut::<CssRuleListStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        record.rules = rules;
    }
    refresh(scope, object, old_length);
    true
}

pub(crate) fn insert(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    index: usize,
    rule: v8::Local<'_, v8::Object>,
) -> bool {
    let Some(old_length) = record(scope, object).map(|record| record.rules.len()) else {
        return false;
    };
    if index > old_length {
        return false;
    }
    let rule = v8::Global::new(scope, rule);
    if let Some(record) = scope
        .get_slot_mut::<CssRuleListStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        record.rules.insert(index, rule);
    }
    refresh(scope, object, old_length);
    true
}

pub(crate) fn delete(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    index: usize,
) -> bool {
    let Some(old_length) = record(scope, object).map(|record| record.rules.len()) else {
        return false;
    };
    if index >= old_length {
        return false;
    }
    if let Some(record) = scope
        .get_slot_mut::<CssRuleListStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        record.rules.remove(index);
    }
    refresh(scope, object, old_length);
    true
}

fn get_length(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new_from_unsigned(scope, record.rules.len() as u32).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn item(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let index = arguments.get(0).uint32_value(scope).unwrap_or(u32::MAX) as usize;
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(rule) = record.rules.get(index) {
        result.set(v8::Local::new(scope, rule).into());
    } else {
        result.set(v8::null(scope).into());
    }
}

fn iterator(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let array = v8::Array::new(scope, record.rules.len() as i32);
    for (index, rule) in record.rules.iter().enumerate() {
        let rule = v8::Local::new(scope, rule);
        let _ = array.set_index(scope, index as u32, rule.into());
    }
    let key = v8::Symbol::get_iterator(scope);
    let function = array
        .get(scope, key.into())
        .and_then(|value| v8::Local::<v8::Function>::try_from(value).ok());
    if let Some(function) = function
        && let Some(iterator) = function.call(scope, array.into(), &[])
    {
        result.set(iterator);
    }
}
