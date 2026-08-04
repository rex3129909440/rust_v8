use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct AttrStore {
    constructors: HashMap<i32, v8::Global<v8::Function>>,
    records: HashMap<i32, AttrRecord>,
}

#[derive(Clone)]
pub(crate) struct AttrRecord {
    pub namespace_uri: Option<String>,
    pub prefix: Option<String>,
    pub local_name: String,
    pub name: String,
    pub value: String,
    pub owner_element: Option<v8::Global<v8::Object>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(AttrStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "Attr", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<AttrStore>()
        .and_then(|store| store.constructors.get(&crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let parent = super::node::ensure_constructor(scope)?;
    let constructor = crate::webidl::create_function(
        scope,
        "Attr",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::attr_namespace_uri_property::define(scope, prototype)?;
    super::attr_prefix_property::define(scope, prototype)?;
    super::attr_local_name_property::define(scope, prototype)?;
    super::attr_name_property::define(scope, prototype)?;
    super::attr_value_property::define(scope, prototype)?;
    super::attr_owner_element_property::define(scope, prototype)?;
    super::attr_specified_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<AttrStore>()
        .ok_or_else(|| "Attr state was not prepared".to_owned())?
        .constructors
        .insert(realm_id, stored);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    name: String,
    value: String,
    namespace_uri: Option<String>,
    owner_element: Option<v8::Local<'_, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create Attr".to_owned());
    }
    let (prefix, local_name) = split_name(&name);
    super::node::attach(scope, object, 2, name.clone(), Some(value.clone()));
    let owner_element = owner_element.map(|element| v8::Global::new(scope, element));
    let stored_record = AttrRecord {
        namespace_uri,
        prefix,
        local_name,
        name,
        value,
        owner_element,
    };
    scope
        .get_slot_mut::<AttrStore>()
        .ok_or_else(|| "Attr state was not prepared".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), stored_record);
    Ok(object)
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<AttrRecord> {
    scope
        .get_slot::<AttrStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn set_owner(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    owner: Option<v8::Local<'_, v8::Object>>,
) -> bool {
    let owner = owner.map(|element| v8::Global::new(scope, element));
    let Some(record) = scope
        .get_slot_mut::<AttrStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    else {
        return false;
    };
    record.owner_element = owner;
    true
}

pub(crate) fn set_stored_value(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    value: String,
) -> bool {
    super::node::set_stored_node_value(scope, object, Some(value.clone()));
    let Some(record) = scope
        .get_slot_mut::<AttrStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    else {
        return false;
    };
    record.value = value;
    true
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Failed to construct 'Attr': Illegal constructor");
}

fn split_name(name: &str) -> (Option<String>, String) {
    match name.split_once(':') {
        Some((prefix, local_name)) => (Some(prefix.to_owned()), local_name.to_owned()),
        None => (None, name.to_owned()),
    }
}
