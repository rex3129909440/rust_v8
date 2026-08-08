use std::collections::HashMap;

#[derive(Clone)]
struct CssViewTransitionRuleRecord {
    navigation: String,
    types: v8::Global<v8::Array>,
}

#[derive(Default)]
pub(crate) struct CssViewTransitionRuleStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, CssViewTransitionRuleRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CssViewTransitionRuleStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "CSSViewTransitionRule", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<CssViewTransitionRuleStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "CSSViewTransitionRule",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "navigation", get_navigation)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "types", get_types)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::css_rule::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<CssViewTransitionRuleStore>()
        .ok_or_else(|| "CSSViewTransitionRule state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

#[allow(dead_code)]
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    navigation: String,
    type_names: Vec<String>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create CSSViewTransitionRule".to_owned());
    }
    let types = v8::Array::new(scope, type_names.len() as i32);
    for (index, value) in type_names.into_iter().enumerate() {
        if let Some(value) = v8::String::new(scope, &value) {
            let _ = types.set_index(scope, index as u32, value.into());
        }
    }
    let types = v8::Global::new(scope, types);
    scope
        .get_slot_mut::<CssViewTransitionRuleStore>()
        .ok_or_else(|| "CSSViewTransitionRule state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            CssViewTransitionRuleRecord { navigation, types },
        );
    Ok(object)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor");
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<CssViewTransitionRuleRecord> {
    scope
        .get_slot::<CssViewTransitionRuleStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_navigation(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, &record.navigation) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_types(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.types).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
