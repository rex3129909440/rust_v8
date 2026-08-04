use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct NamedNodeMapStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, NamedNodeMapRecord>,
    maps_by_element: HashMap<i32, v8::Global<v8::Object>>,
}

#[derive(Clone)]
struct NamedNodeMapRecord {
    element: v8::Global<v8::Object>,
    attributes: Vec<v8::Global<v8::Object>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(NamedNodeMapStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "NamedNodeMap", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<NamedNodeMapStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "NamedNodeMap",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::named_node_map_length_property::define(scope, prototype)?;
    super::named_node_map_get_named_item::define(scope, prototype)?;
    super::named_node_map_get_named_item_ns::define(scope, prototype)?;
    super::named_node_map_item::define(scope, prototype)?;
    super::named_node_map_remove_named_item::define(scope, prototype)?;
    super::named_node_map_remove_named_item_ns::define(scope, prototype)?;
    super::named_node_map_set_named_item::define(scope, prototype)?;
    super::named_node_map_set_named_item_ns::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    super::named_node_map_values_iterator::define(scope, prototype)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<NamedNodeMapStore>()
        .ok_or_else(|| "NamedNodeMap state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create_for_element<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    element: v8::Local<'_, v8::Object>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let element_id = element.get_identity_hash().get();
    let existing = scope
        .get_slot::<NamedNodeMapStore>()
        .and_then(|store| store.maps_by_element.get(&element_id))
        .cloned();
    if let Some(existing) = existing {
        let map = v8::Local::new(scope, &existing);
        synchronize(scope, map, element)?;
        return Ok(map);
    }
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let map = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, map, prototype.into()) != Some(true) {
        return Err("cannot create NamedNodeMap".to_owned());
    }
    let element_global = v8::Global::new(scope, element);
    let map_global = v8::Global::new(scope, map);
    let store = scope
        .get_slot_mut::<NamedNodeMapStore>()
        .ok_or_else(|| "NamedNodeMap state was not prepared".to_owned())?;
    store.records.insert(
        map.get_identity_hash().get(),
        NamedNodeMapRecord {
            element: element_global,
            attributes: Vec::new(),
        },
    );
    store.maps_by_element.insert(element_id, map_global);
    synchronize(scope, map, element)?;
    Ok(map)
}

pub(crate) fn sync_existing(scope: &mut v8::PinScope<'_, '_>, element: v8::Local<'_, v8::Object>) {
    let existing = scope
        .get_slot::<NamedNodeMapStore>()
        .and_then(|store| {
            store
                .maps_by_element
                .get(&element.get_identity_hash().get())
        })
        .cloned();
    if let Some(existing) = existing {
        let map = v8::Local::new(scope, &existing);
        let _ = synchronize(scope, map, element);
    }
}

fn synchronize(
    scope: &mut v8::PinScope<'_, '_>,
    map: v8::Local<'_, v8::Object>,
    element: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    let snapshots = super::element::attributes_snapshot(scope, element)
        .ok_or_else(|| "NamedNodeMap owner is not an Element".to_owned())?;
    let previous = map_record(scope, map)
        .map(|record| record.attributes)
        .unwrap_or_default();
    remove_own_attribute_properties(scope, map, &previous);
    let mut next = Vec::new();
    for snapshot in snapshots {
        let existing = previous.iter().find_map(|attribute| {
            let attribute = v8::Local::new(scope, attribute);
            let record = super::attr::record(scope, attribute)?;
            (record.name.eq_ignore_ascii_case(&snapshot.name)
                && record.namespace_uri == snapshot.namespace_uri)
                .then_some(attribute)
        });
        let attribute = match existing {
            Some(attribute) => {
                super::attr::set_stored_value(scope, attribute, snapshot.value);
                super::attr::set_owner(scope, attribute, Some(element));
                attribute
            }
            None => super::attr::create(
                scope,
                snapshot.name,
                snapshot.value,
                snapshot.namespace_uri,
                Some(element),
            )?,
        };
        next.push(v8::Global::new(scope, attribute));
    }
    for attribute in &previous {
        let attribute = v8::Local::new(scope, attribute);
        let retained = next.iter().any(|candidate| {
            v8::Local::new(scope, candidate).get_identity_hash().get()
                == attribute.get_identity_hash().get()
        });
        if !retained {
            super::attr::set_owner(scope, attribute, None);
        }
    }
    define_own_attribute_properties(scope, map, &next);
    if let Some(record) = scope
        .get_slot_mut::<NamedNodeMapStore>()
        .and_then(|store| store.records.get_mut(&map.get_identity_hash().get()))
    {
        record.attributes = next;
    }
    Ok(())
}

fn remove_own_attribute_properties(
    scope: &v8::PinScope<'_, '_>,
    map: v8::Local<'_, v8::Object>,
    attributes: &[v8::Global<v8::Object>],
) {
    for (index, attribute) in attributes.iter().enumerate() {
        if let Some(key) = v8::String::new(scope, &index.to_string()) {
            let _ = map.delete(scope, key.into());
        }
        let attribute = v8::Local::new(scope, attribute);
        if let Some(record) = super::attr::record(scope, attribute)
            && let Some(key) = v8::String::new(scope, &record.name)
        {
            let _ = map.delete(scope, key.into());
        }
    }
}

fn define_own_attribute_properties(
    scope: &v8::PinScope<'_, '_>,
    map: v8::Local<'_, v8::Object>,
    attributes: &[v8::Global<v8::Object>],
) {
    for (index, attribute) in attributes.iter().enumerate() {
        let attribute = v8::Local::new(scope, attribute);
        if let Some(index_key) = v8::String::new(scope, &index.to_string()) {
            let _ = map.define_own_property(
                scope,
                index_key.into(),
                attribute.into(),
                v8::PropertyAttribute::READ_ONLY,
            );
        }
        if let Some(record) = super::attr::record(scope, attribute)
            && let Some(name_key) = v8::String::new(scope, &record.name)
        {
            let _ = map.define_own_property(
                scope,
                name_key.into(),
                attribute.into(),
                v8::PropertyAttribute::READ_ONLY,
            );
        }
    }
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'NamedNodeMap': Illegal constructor",
    );
}

fn map_record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<NamedNodeMapRecord> {
    scope
        .get_slot::<NamedNodeMapStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn attributes<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<Vec<v8::Local<'s, v8::Object>>> {
    map_record(scope, object).map(|record| {
        record
            .attributes
            .iter()
            .map(|attribute| v8::Local::new(scope, attribute))
            .collect()
    })
}

fn owner<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<v8::Local<'s, v8::Object>> {
    map_record(scope, object).map(|record| v8::Local::new(scope, &record.element))
}

pub(crate) fn return_match(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    mut result: v8::ReturnValue<'_>,
    matches: impl Fn(&super::attr::AttrRecord) -> bool,
) {
    let Some(attributes) = attributes(scope, object) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(attribute) = attributes
        .into_iter()
        .find(|attribute| super::attr::record(scope, *attribute).is_some_and(|r| matches(&r)))
    {
        result.set(attribute.into());
    } else {
        result.set(v8::null(scope).into());
    }
}

pub(crate) fn set_item(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    namespace_aware: bool,
) {
    let Ok(attribute) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "The supplied node is not an Attr");
        return;
    };
    let Some(attribute_record) = super::attr::record(scope, attribute) else {
        throw_dom_exception(
            scope,
            "HierarchyRequestError",
            "The supplied node is not an Attr",
        );
        return;
    };
    let Some(element) = owner(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(current_owner) = attribute_record.owner_element.as_ref() {
        let current_owner = v8::Local::new(scope, current_owner);
        if current_owner.get_identity_hash().get() != element.get_identity_hash().get() {
            throw_dom_exception(
                scope,
                "InUseAttributeError",
                "The attribute is already in use by another element",
            );
            return;
        }
    }
    let old = attributes(scope, arguments.this()).and_then(|attributes| {
        attributes.into_iter().find(|candidate| {
            super::attr::record(scope, *candidate).is_some_and(|candidate| {
                if namespace_aware {
                    candidate.namespace_uri == attribute_record.namespace_uri
                        && candidate.local_name == attribute_record.local_name
                } else {
                    candidate.name.eq_ignore_ascii_case(&attribute_record.name)
                }
            })
        })
    });
    super::element::set_attribute_full(
        scope,
        element,
        attribute_record.name.clone(),
        attribute_record.value.clone(),
        attribute_record.namespace_uri.clone(),
    );
    replace_attribute_object(
        scope,
        arguments.this(),
        attribute_record.name.clone(),
        attribute,
    );
    super::attr::set_owner(scope, attribute, Some(element));
    if let Some(old) = old {
        if old.get_identity_hash().get() != attribute.get_identity_hash().get() {
            super::attr::set_owner(scope, old, None);
        }
        result.set(old.into());
    } else {
        result.set(v8::null(scope).into());
    }
}

fn replace_attribute_object(
    scope: &mut v8::PinScope<'_, '_>,
    map: v8::Local<'_, v8::Object>,
    name: String,
    replacement: v8::Local<'_, v8::Object>,
) {
    let previous = map_record(scope, map)
        .map(|record| record.attributes)
        .unwrap_or_default();
    remove_own_attribute_properties(scope, map, &previous);
    let mut next = previous;
    if let Some(index) = next.iter().position(|candidate| {
        let candidate = v8::Local::new(scope, candidate);
        super::attr::record(scope, candidate)
            .is_some_and(|record| record.name.eq_ignore_ascii_case(&name))
    }) {
        next[index] = v8::Global::new(scope, replacement);
    } else {
        next.push(v8::Global::new(scope, replacement));
    }
    define_own_attribute_properties(scope, map, &next);
    if let Some(record) = scope
        .get_slot_mut::<NamedNodeMapStore>()
        .and_then(|store| store.records.get_mut(&map.get_identity_hash().get()))
    {
        record.attributes = next;
    }
}

pub(crate) fn remove_item(
    scope: &mut v8::PinScope<'_, '_>,
    map: v8::Local<'_, v8::Object>,
    mut result: v8::ReturnValue<'_>,
    namespace: Option<Option<String>>,
    name: &str,
) {
    let Some(element) = owner(scope, map) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let old = attributes(scope, map).and_then(|attributes| {
        attributes.into_iter().find(|attribute| {
            super::attr::record(scope, *attribute).is_some_and(|record| match &namespace {
                Some(namespace) => &record.namespace_uri == namespace && record.local_name == name,
                None => record.name.eq_ignore_ascii_case(name),
            })
        })
    });
    let Some(old) = old else {
        throw_dom_exception(
            scope,
            "NotFoundError",
            "No attribute with the requested name exists",
        );
        return;
    };
    let record = super::attr::record(scope, old).expect("Attr record");
    super::element::remove_attribute_full(
        scope,
        element,
        record.namespace_uri.as_deref(),
        &record.local_name,
    );
    super::attr::set_owner(scope, old, None);
    result.set(old.into());
}

pub(crate) fn optional_namespace(
    scope: &v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> Option<String> {
    if value.is_null() {
        None
    } else {
        Some(crate::webidl::value_to_string(scope, value))
    }
}

fn throw_dom_exception(scope: &mut v8::PinScope<'_, '_>, name: &str, message: &str) {
    match super::dom_exception::create(scope, message.to_owned(), name.to_owned()) {
        Ok(exception) => {
            scope.throw_exception(exception.into());
        }
        Err(error) => crate::webidl::throw_type_error(scope, &error),
    }
}
