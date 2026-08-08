use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct HtmlCollectionStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, HtmlCollectionRecord>,
    form_owners: HashMap<i32, v8::Global<v8::Object>>,
    data_list_owners: HashMap<i32, v8::Global<v8::Object>>,
}

#[derive(Clone)]
enum HtmlCollectionRecord {
    Snapshot(Vec<v8::Global<v8::Object>>),
    Live {
        root: v8::Global<v8::Object>,
        query: HtmlCollectionQuery,
        materialized_length: usize,
    },
}

#[derive(Clone)]
pub(crate) enum HtmlCollectionQuery {
    Children,
    TagName(String),
    TagNameNs {
        namespace: Option<String>,
        local_name: String,
    },
    ClassNames(Vec<String>),
    Name(String),
    Legacy(String),
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(HtmlCollectionStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "HTMLCollection", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<HtmlCollectionStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "HTMLCollection",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::html_collection_length_property::define(scope, prototype)?;
    super::html_collection_item::define(scope, prototype)?;
    super::html_collection_named_item::define(scope, prototype)?;
    super::html_collection_values_iterator::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::move_iterator_to_end(scope, prototype)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<HtmlCollectionStore>()
        .ok_or_else(|| "HTMLCollection state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    items: Vec<v8::Local<'_, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let collection = new_exotic_collection(scope)?;
    if crate::webidl::set_platform_prototype(scope, collection, prototype.into()) != Some(true) {
        return Err("cannot create HTMLCollection".to_owned());
    }
    let values = items
        .iter()
        .map(|item| v8::Global::new(scope, *item))
        .collect::<Vec<_>>();
    scope
        .get_slot_mut::<HtmlCollectionStore>()
        .ok_or_else(|| "HTMLCollection state was not prepared".to_owned())?
        .records
        .insert(
            collection.get_identity_hash().get(),
            HtmlCollectionRecord::Snapshot(values),
        );
    Ok(collection)
}

pub(crate) fn create_live<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    root: v8::Local<'_, v8::Object>,
    query: HtmlCollectionQuery,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let collection = new_exotic_collection(scope)?;
    if crate::webidl::set_platform_prototype(scope, collection, prototype.into()) != Some(true) {
        return Err("cannot create live HTMLCollection".to_owned());
    }
    let initial = resolve_query(scope, root, &query);
    let root = v8::Global::new(scope, root);
    scope
        .get_slot_mut::<HtmlCollectionStore>()
        .ok_or_else(|| "HTMLCollection state was not prepared".to_owned())?
        .records
        .insert(
            collection.get_identity_hash().get(),
            HtmlCollectionRecord::Live {
                root,
                query,
                materialized_length: initial.len(),
            },
        );
    Ok(collection)
}

fn new_exotic_collection<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let template = v8::ObjectTemplate::new(scope);
    template.set_indexed_property_handler(
        v8::IndexedPropertyHandlerConfiguration::new()
            .getter(indexed_getter)
            .query(indexed_query)
            .enumerator(indexed_enumerator),
    );
    template.set_named_property_handler(
        v8::NamedPropertyHandlerConfiguration::new()
            .getter(named_getter)
            .query(named_query)
            .enumerator(named_enumerator),
    );
    template
        .new_instance(scope)
        .ok_or_else(|| "cannot create HTMLCollection exotic object".to_owned())
}

pub(crate) fn replace(
    scope: &mut v8::PinScope<'_, '_>,
    collection: v8::Local<'_, v8::Object>,
    items: Vec<v8::Local<'_, v8::Object>>,
) -> bool {
    let identity = collection.get_identity_hash().get();
    let Some(old_length) = scope
        .get_slot::<HtmlCollectionStore>()
        .and_then(|store| store.records.get(&identity))
        .map(|record| match record {
            HtmlCollectionRecord::Snapshot(values) => values.len(),
            HtmlCollectionRecord::Live {
                materialized_length,
                ..
            } => *materialized_length,
        })
    else {
        return false;
    };
    let values = items
        .iter()
        .map(|item| v8::Global::new(scope, *item))
        .collect::<Vec<_>>();
    scope
        .get_slot_mut::<HtmlCollectionStore>()
        .and_then(|store| {
            store
                .records
                .insert(identity, HtmlCollectionRecord::Snapshot(values))
        })
        .is_some()
        || old_length == 0
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor");
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<Vec<v8::Global<v8::Object>>> {
    let record = scope
        .get_slot::<HtmlCollectionStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()?;
    match record {
        HtmlCollectionRecord::Snapshot(values) => Some(values),
        HtmlCollectionRecord::Live { root, query, .. } => {
            let root = v8::Local::new(scope, &root);
            Some(
                resolve_query(scope, root, &query)
                    .into_iter()
                    .map(|item| v8::Global::new(scope, item))
                    .collect(),
            )
        }
    }
}

pub(crate) fn items(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<Vec<v8::Global<v8::Object>>> {
    record(scope, object)
}

pub(crate) fn register_form_owner(
    scope: &mut v8::PinScope<'_, '_>,
    collection: v8::Local<'_, v8::Object>,
    form: v8::Local<'_, v8::Object>,
) -> bool {
    let owner = v8::Global::new(scope, form);
    if let Some(store) = scope.get_slot_mut::<HtmlCollectionStore>() {
        store
            .form_owners
            .insert(collection.get_identity_hash().get(), owner);
        true
    } else {
        false
    }
}

pub(crate) fn register_data_list_owner(
    scope: &mut v8::PinScope<'_, '_>,
    collection: v8::Local<'_, v8::Object>,
    data_list: v8::Local<'_, v8::Object>,
) -> bool {
    let owner = v8::Global::new(scope, data_list);
    if let Some(store) = scope.get_slot_mut::<HtmlCollectionStore>() {
        store
            .data_list_owners
            .insert(collection.get_identity_hash().get(), owner);
        true
    } else {
        false
    }
}

pub(crate) fn refresh_live(
    scope: &mut v8::PinScope<'_, '_>,
    collection: v8::Local<'_, v8::Object>,
) {
    let owner = scope
        .get_slot::<HtmlCollectionStore>()
        .and_then(|store| store.form_owners.get(&collection.get_identity_hash().get()))
        .cloned();
    if let Some(owner) = owner {
        let owner = v8::Local::new(scope, &owner);
        let controls = super::html_form_element::collect_controls(scope, owner);
        replace(scope, collection, controls);
        return;
    }
    let owner = scope
        .get_slot::<HtmlCollectionStore>()
        .and_then(|store| {
            store
                .data_list_owners
                .get(&collection.get_identity_hash().get())
        })
        .cloned();
    if let Some(owner) = owner {
        let owner = v8::Local::new(scope, &owner);
        let options = super::html_data_list_element::collect_options(scope, owner);
        replace(scope, collection, options);
        return;
    }
    let live = scope
        .get_slot::<HtmlCollectionStore>()
        .and_then(|store| store.records.get(&collection.get_identity_hash().get()))
        .cloned();
    if let Some(HtmlCollectionRecord::Live {
        root,
        query,
        materialized_length: _,
    }) = live
    {
        let root = v8::Local::new(scope, &root);
        let items = resolve_query(scope, root, &query);
        if let Some(HtmlCollectionRecord::Live {
            materialized_length,
            ..
        }) = scope
            .get_slot_mut::<HtmlCollectionStore>()
            .and_then(|store| store.records.get_mut(&collection.get_identity_hash().get()))
        {
            *materialized_length = items.len();
        }
    }
}

fn resolve_query<'s>(
    scope: &v8::PinScope<'s, '_>,
    root: v8::Local<'s, v8::Object>,
    query: &HtmlCollectionQuery,
) -> Vec<v8::Local<'s, v8::Object>> {
    if matches!(query, HtmlCollectionQuery::Children) {
        return super::node::children(scope, root)
            .into_iter()
            .filter(|child| super::element::record(scope, *child).is_some())
            .collect();
    }
    super::dom_selector::descendants(scope, root)
        .into_iter()
        .filter(|element| {
            let Some(record) = super::element::record(scope, *element) else {
                return false;
            };
            match query {
                HtmlCollectionQuery::Children => false,
                HtmlCollectionQuery::TagName(name) => {
                    name == "*" || record.tag_name.eq_ignore_ascii_case(name)
                }
                HtmlCollectionQuery::TagNameNs {
                    namespace,
                    local_name,
                } => {
                    (namespace.as_deref() == Some("*")
                        || record.namespace_uri.as_deref() == namespace.as_deref())
                        && (local_name == "*"
                            || record
                                .tag_name
                                .rsplit(':')
                                .next()
                                .unwrap_or(&record.tag_name)
                                .eq_ignore_ascii_case(local_name))
                }
                HtmlCollectionQuery::ClassNames(names) => {
                    let classes = super::element::attribute_value(scope, *element, "class")
                        .unwrap_or_default();
                    names.iter().all(|wanted| {
                        classes
                            .split_ascii_whitespace()
                            .any(|token| token == wanted)
                    })
                }
                HtmlCollectionQuery::Name(name) => {
                    super::element::attribute_value(scope, *element, "name").as_deref()
                        == Some(name)
                }
                HtmlCollectionQuery::Legacy(kind) => match kind.as_str() {
                    "images" => record.tag_name.eq_ignore_ascii_case("IMG"),
                    "embeds" | "plugins" => record.tag_name.eq_ignore_ascii_case("EMBED"),
                    "links" => {
                        (record.tag_name.eq_ignore_ascii_case("A")
                            || record.tag_name.eq_ignore_ascii_case("AREA"))
                            && super::element::attribute_value(scope, *element, "href").is_some()
                    }
                    "forms" => record.tag_name.eq_ignore_ascii_case("FORM"),
                    "scripts" => record.tag_name.eq_ignore_ascii_case("SCRIPT"),
                    "anchors" => {
                        record.tag_name.eq_ignore_ascii_case("A")
                            && super::element::attribute_value(scope, *element, "name").is_some()
                    }
                    _ => false,
                },
            }
        })
        .collect()
}

fn indexed_getter(
    scope: &mut v8::PinScope<'_, '_>,
    index: u32,
    arguments: v8::PropertyCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_, v8::Value>,
) -> v8::Intercepted {
    crate::trace::record_indexed_native_intercept(scope, &arguments, "get", index, None);
    let Some(values) = record(scope, arguments.holder()) else {
        return v8::Intercepted::kNo;
    };
    let Some(value) = values.get(index as usize) else {
        return v8::Intercepted::kNo;
    };
    result.set(v8::Local::new(scope, value).into());
    v8::Intercepted::kYes
}

fn indexed_query(
    scope: &mut v8::PinScope<'_, '_>,
    index: u32,
    arguments: v8::PropertyCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_, v8::Integer>,
) -> v8::Intercepted {
    crate::trace::record_indexed_native_intercept(scope, &arguments, "has", index, None);
    if record(scope, arguments.holder()).is_some_and(|values| (index as usize) < values.len()) {
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
    let length = record(scope, arguments.holder()).map_or(0, |values| values.len());
    let indices = (0..length)
        .map(|index| v8::Integer::new_from_unsigned(scope, index as u32).into())
        .collect::<Vec<v8::Local<v8::Value>>>();
    result.set(v8::Array::new_with_elements(scope, &indices));
}

fn named_getter(
    scope: &mut v8::PinScope<'_, '_>,
    key: v8::Local<'_, v8::Name>,
    arguments: v8::PropertyCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_, v8::Value>,
) -> v8::Intercepted {
    crate::trace::record_named_native_intercept(scope, &arguments, "get", key, None);
    let Some(name) = property_name(scope, key) else {
        return v8::Intercepted::kNo;
    };
    let Some(value) = named_value(scope, arguments.holder(), &name) else {
        return v8::Intercepted::kNo;
    };
    result.set(value.into());
    v8::Intercepted::kYes
}

fn named_query(
    scope: &mut v8::PinScope<'_, '_>,
    key: v8::Local<'_, v8::Name>,
    arguments: v8::PropertyCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_, v8::Integer>,
) -> v8::Intercepted {
    crate::trace::record_named_native_intercept(scope, &arguments, "has", key, None);
    let Some(name) = property_name(scope, key) else {
        return v8::Intercepted::kNo;
    };
    if named_value(scope, arguments.holder(), &name).is_some() {
        result.set_int32(1);
        v8::Intercepted::kYes
    } else {
        v8::Intercepted::kNo
    }
}

fn named_enumerator(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::PropertyCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_, v8::Array>,
) {
    crate::trace::record_native_enumeration(scope, &arguments);
    let Some(values) = record(scope, arguments.holder()) else {
        result.set(v8::Array::new(scope, 0));
        return;
    };
    let mut names = Vec::new();
    for value in values {
        let value = v8::Local::new(scope, &value);
        for attribute in ["id", "name"] {
            let Some(name) = super::element::attribute_value(scope, value, attribute) else {
                continue;
            };
            if !name.is_empty() && !names.contains(&name) {
                names.push(name);
            }
        }
    }
    let names = names
        .iter()
        .filter_map(|name| v8::String::new(scope, name).map(|name| name.into()))
        .collect::<Vec<v8::Local<v8::Value>>>();
    result.set(v8::Array::new_with_elements(scope, &names));
}

fn property_name(scope: &v8::PinScope<'_, '_>, key: v8::Local<'_, v8::Name>) -> Option<String> {
    if key.is_symbol() {
        return None;
    }
    key.to_string(scope)
        .map(|value| value.to_rust_string_lossy(scope))
}

fn named_value<'s>(
    scope: &v8::PinScope<'s, '_>,
    collection: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<v8::Local<'s, v8::Object>> {
    if matches!(
        name,
        "length" | "item" | "namedItem" | "constructor" | "__proto__"
    ) {
        return None;
    }
    record(scope, collection)?.into_iter().find_map(|item| {
        let item = v8::Local::new(scope, &item);
        let matched = ["id", "name"].into_iter().any(|attribute| {
            super::element::attribute_value(scope, item, attribute).as_deref() == Some(name)
        });
        matched.then_some(item)
    })
}
