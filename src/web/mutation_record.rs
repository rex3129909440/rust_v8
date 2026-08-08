use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct MutationRecordStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, MutationRecordData>,
}

#[derive(Clone)]
pub(crate) struct MutationRecordData {
    pub(crate) kind: String,
    pub(crate) target: v8::Global<v8::Object>,
    pub(crate) added_nodes: v8::Global<v8::Object>,
    pub(crate) removed_nodes: v8::Global<v8::Object>,
    pub(crate) previous_sibling: Option<v8::Global<v8::Object>>,
    pub(crate) next_sibling: Option<v8::Global<v8::Object>>,
    pub(crate) attribute_name: Option<String>,
    pub(crate) attribute_namespace: Option<String>,
    pub(crate) old_value: Option<String>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(MutationRecordStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "MutationRecord", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<MutationRecordStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "MutationRecord",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::mutation_record_type_property::define(scope, prototype)?;
    super::mutation_record_target_property::define(scope, prototype)?;
    super::mutation_record_added_nodes_property::define(scope, prototype)?;
    super::mutation_record_removed_nodes_property::define(scope, prototype)?;
    super::mutation_record_previous_sibling_property::define(scope, prototype)?;
    super::mutation_record_next_sibling_property::define(scope, prototype)?;
    super::mutation_record_attribute_name_property::define(scope, prototype)?;
    super::mutation_record_attribute_namespace_property::define(scope, prototype)?;
    super::mutation_record_old_value_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<MutationRecordStore>()
        .ok_or_else(|| "MutationRecord state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create_attribute<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    target: v8::Local<'_, v8::Object>,
    attribute_name: String,
    attribute_namespace: Option<String>,
    old_value: Option<String>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    create(
        scope,
        "attributes",
        target,
        Vec::new(),
        Vec::new(),
        None,
        None,
        Some(attribute_name),
        attribute_namespace,
        old_value,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    kind: &str,
    target: v8::Local<'_, v8::Object>,
    added_nodes: Vec<v8::Local<'_, v8::Object>>,
    removed_nodes: Vec<v8::Local<'_, v8::Object>>,
    previous_sibling: Option<v8::Local<'_, v8::Object>>,
    next_sibling: Option<v8::Local<'_, v8::Object>>,
    attribute_name: Option<String>,
    attribute_namespace: Option<String>,
    old_value: Option<String>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create MutationRecord".to_owned());
    }
    let added_nodes = super::node_list::create(scope, added_nodes)?;
    let removed_nodes = super::node_list::create(scope, removed_nodes)?;
    let data = MutationRecordData {
        kind: kind.to_owned(),
        target: v8::Global::new(scope, target),
        added_nodes: v8::Global::new(scope, added_nodes),
        removed_nodes: v8::Global::new(scope, removed_nodes),
        previous_sibling: previous_sibling.map(|sibling| v8::Global::new(scope, sibling)),
        next_sibling: next_sibling.map(|sibling| v8::Global::new(scope, sibling)),
        attribute_name,
        attribute_namespace,
        old_value,
    };
    scope
        .get_slot_mut::<MutationRecordStore>()
        .ok_or_else(|| "MutationRecord state was not prepared".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), data);
    Ok(object)
}

pub(crate) fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'MutationRecord': Illegal constructor",
    );
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<MutationRecordData> {
    scope
        .get_slot::<MutationRecordStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn get_optional_object(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&MutationRecordData) -> Option<&v8::Global<v8::Object>>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(object) = select(&record) {
        result.set(v8::Local::new(scope, object).into());
    } else {
        result.set(v8::null(scope).into());
    }
}

pub(crate) fn get_optional_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&MutationRecordData) -> Option<&str>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(value) = select(&record)
        && let Some(value) = v8::String::new(scope, value)
    {
        result.set(value.into());
    } else {
        result.set(v8::null(scope).into());
    }
}
