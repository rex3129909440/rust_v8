use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct NodeListStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, NodeListRecord>,
}

#[derive(Clone)]
enum NodeListRecord {
    Snapshot(Vec<v8::Global<v8::Object>>),
    ChildNodes(v8::Global<v8::Object>),
    Labels(v8::Global<v8::Object>),
    NamedElements {
        root: v8::Global<v8::Object>,
        name: String,
    },
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(NodeListStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "NodeList", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<NodeListStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "NodeList",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::node_list_entries::define(scope, prototype)?;
    super::node_list_keys::define(scope, prototype)?;
    super::node_list_values::define(scope, prototype)?;
    super::node_list_for_each::define(scope, prototype)?;
    super::node_list_length_property::define(scope, prototype)?;
    super::node_list_item::define(scope, prototype)?;
    let values_key = crate::webidl::string(scope, "values")?;
    let values_method = prototype
        .get(scope, values_key.into())
        .ok_or_else(|| "NodeList.values is unavailable".to_owned())?;
    let iterator = v8::Symbol::get_iterator(scope);
    if prototype.define_own_property(
        scope,
        iterator.into(),
        values_method,
        v8::PropertyAttribute::NONE,
    ) != Some(true)
    {
        return Err("cannot define NodeList iterator".to_owned());
    }
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::move_iterator_to_end(scope, prototype)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<NodeListStore>()
        .ok_or_else(|| "NodeList state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    items: Vec<v8::Local<'_, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    create_with_constructor(scope, constructor, items)
}

pub(crate) fn create_with_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    constructor: v8::Local<'_, v8::Function>,
    items: Vec<v8::Local<'_, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let list = new_exotic_list(scope)?;
    if crate::webidl::set_platform_prototype(scope, list, prototype.into()) != Some(true) {
        return Err("cannot create NodeList".to_owned());
    }
    let values = items
        .into_iter()
        .map(|item| v8::Global::new(scope, item))
        .collect();
    scope
        .get_slot_mut::<NodeListStore>()
        .ok_or_else(|| "NodeList state was not prepared".to_owned())?
        .records
        .insert(
            list.get_identity_hash().get(),
            NodeListRecord::Snapshot(values),
        );
    Ok(list)
}

pub(crate) fn replace_snapshot(
    scope: &mut v8::PinScope<'_, '_>,
    list: v8::Local<'_, v8::Object>,
    items: Vec<v8::Local<'_, v8::Object>>,
) -> bool {
    let values = items
        .into_iter()
        .map(|item| v8::Global::new(scope, item))
        .collect();
    let Some(record) = scope
        .get_slot_mut::<NodeListStore>()
        .and_then(|store| store.records.get_mut(&list.get_identity_hash().get()))
    else {
        return false;
    };
    *record = NodeListRecord::Snapshot(values);
    true
}

pub(crate) fn create_live_child_nodes<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    owner: v8::Local<'_, v8::Object>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let list = new_exotic_list(scope)?;
    if crate::webidl::set_platform_prototype(scope, list, prototype.into()) != Some(true) {
        return Err("cannot create live NodeList".to_owned());
    }
    let owner = v8::Global::new(scope, owner);
    scope
        .get_slot_mut::<NodeListStore>()
        .ok_or_else(|| "NodeList state was not prepared".to_owned())?
        .records
        .insert(
            list.get_identity_hash().get(),
            NodeListRecord::ChildNodes(owner),
        );
    Ok(list)
}

pub(crate) fn create_live_named_elements<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    root: v8::Local<'_, v8::Object>,
    name: String,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let list = new_exotic_list(scope)?;
    if crate::webidl::set_platform_prototype(scope, list, prototype.into()) != Some(true) {
        return Err("cannot create live NodeList".to_owned());
    }
    let root = v8::Global::new(scope, root);
    scope
        .get_slot_mut::<NodeListStore>()
        .ok_or_else(|| "NodeList state was not prepared".to_owned())?
        .records
        .insert(
            list.get_identity_hash().get(),
            NodeListRecord::NamedElements { root, name },
        );
    Ok(list)
}

pub(crate) fn register_labels_owner(
    scope: &mut v8::PinScope<'_, '_>,
    list: v8::Local<'_, v8::Object>,
    owner: v8::Local<'_, v8::Object>,
) -> bool {
    let owner = v8::Global::new(scope, owner);
    let Some(record) = scope
        .get_slot_mut::<NodeListStore>()
        .and_then(|store| store.records.get_mut(&list.get_identity_hash().get()))
    else {
        return false;
    };
    *record = NodeListRecord::Labels(owner);
    true
}

fn new_exotic_list<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let template = v8::ObjectTemplate::new(scope);
    template.set_indexed_property_handler(
        v8::IndexedPropertyHandlerConfiguration::new()
            .getter(indexed_getter)
            .query(indexed_query)
            .enumerator(indexed_enumerator),
    );
    template
        .new_instance(scope)
        .ok_or_else(|| "cannot create NodeList exotic object".to_owned())
}

pub(crate) fn items<'s>(
    scope: &v8::PinScope<'s, '_>,
    list: v8::Local<'_, v8::Object>,
) -> Option<Vec<v8::Local<'s, v8::Object>>> {
    let record = scope
        .get_slot::<NodeListStore>()?
        .records
        .get(&list.get_identity_hash().get())
        .cloned()?;
    Some(match record {
        NodeListRecord::Snapshot(items) => items
            .iter()
            .map(|item| v8::Local::new(scope, item))
            .collect(),
        NodeListRecord::ChildNodes(owner) => {
            super::node::children(scope, v8::Local::new(scope, &owner))
        }
        NodeListRecord::Labels(owner) => {
            super::html_label_element::labels_for(scope, v8::Local::new(scope, &owner))
        }
        NodeListRecord::NamedElements { root, name } => {
            let root = v8::Local::new(scope, &root);
            super::dom_selector::descendants(scope, root)
                .into_iter()
                .filter(|element| {
                    super::element::attribute_value(scope, *element, "name").as_deref()
                        == Some(name.as_str())
                })
                .collect()
        }
    })
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Failed to construct 'NodeList': Illegal constructor");
}

pub(crate) fn list_or_throw<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<Vec<v8::Local<'s, v8::Object>>> {
    let values = items(scope, object);
    if values.is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
    values
}

fn indexed_getter(
    scope: &mut v8::PinScope<'_, '_>,
    index: u32,
    arguments: v8::PropertyCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_, v8::Value>,
) -> v8::Intercepted {
    crate::trace::record_indexed_native_intercept(scope, &arguments, "get", index, None);
    let Some(items) = items(scope, arguments.holder()) else {
        return v8::Intercepted::kNo;
    };
    let Some(item) = items.get(index as usize) else {
        return v8::Intercepted::kNo;
    };
    result.set((*item).into());
    v8::Intercepted::kYes
}

fn indexed_query(
    scope: &mut v8::PinScope<'_, '_>,
    index: u32,
    arguments: v8::PropertyCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_, v8::Integer>,
) -> v8::Intercepted {
    crate::trace::record_indexed_native_intercept(scope, &arguments, "has", index, None);
    if items(scope, arguments.holder()).is_some_and(|items| (index as usize) < items.len()) {
        result.set_int32(1);
        v8::Intercepted::kYes
    } else {
        v8::Intercepted::kNo
    }
}

fn indexed_enumerator(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::PropertyCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_, v8::Array>,
) {
    crate::trace::record_native_enumeration(scope, &arguments);
    let length = items(scope, arguments.holder()).map_or(0, |items| items.len());
    let indices = (0..length)
        .map(|index| v8::Integer::new_from_unsigned(scope, index as u32).into())
        .collect::<Vec<v8::Local<v8::Value>>>();
    result.set(v8::Array::new_with_elements(scope, &indices));
}

pub(crate) fn return_iterator(
    scope: &mut v8::PinScope<'_, '_>,
    array: v8::Local<'_, v8::Array>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(key) = v8::String::new(scope, "values") else {
        return;
    };
    let Some(function) = array
        .get(scope, key.into())
        .and_then(|value| v8::Local::<v8::Function>::try_from(value).ok())
    else {
        return;
    };
    if let Some(iterator) = function.call(scope, array.into(), &[]) {
        result.set(iterator);
    }
}
