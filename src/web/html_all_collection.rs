use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct HtmlAllCollectionStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, HtmlAllCollectionRecord>,
}

#[derive(Clone)]
struct HtmlAllCollectionRecord {
    object: v8::Global<v8::Object>,
    document: v8::Global<v8::Object>,
    materialized_length: usize,
    named_keys: Vec<String>,
}

#[cfg(target_env = "msvc")]
unsafe extern "C" {
    #[link_name = "?MarkAsUndetectable@ObjectTemplate@v8@@QEAAXXZ"]
    fn mark_as_undetectable(template: *const v8::ObjectTemplate);

    #[link_name = "?SetCallAsFunctionHandler@ObjectTemplate@v8@@QEAAXP6AXAEBV?$FunctionCallbackInfo@VValue@v8@@@2@@ZV?$Local@VValue@v8@@@2@@Z"]
    fn set_call_as_function_handler(
        template: *const v8::ObjectTemplate,
        callback: v8::FunctionCallback,
        data: v8::Local<'_, v8::Value>,
    );
}

#[cfg(not(target_env = "msvc"))]
unsafe extern "C" {
    #[link_name = "_ZN2v814ObjectTemplate18MarkAsUndetectableEv"]
    fn mark_as_undetectable(template: *const v8::ObjectTemplate);

    #[link_name = "_ZN2v814ObjectTemplate24SetCallAsFunctionHandlerEPFvRKNS_20FunctionCallbackInfoINS_5ValueEEEENS_5LocalIS2_EE"]
    fn set_call_as_function_handler(
        template: *const v8::ObjectTemplate,
        callback: v8::FunctionCallback,
        data: v8::Local<'_, v8::Value>,
    );
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(HtmlAllCollectionStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "HTMLAllCollection", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<HtmlAllCollectionStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "HTMLAllCollection",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "length", get_length)?;
    crate::webidl::define_method(scope, prototype, "item", 0, item)?;
    crate::webidl::define_method(scope, prototype, "namedItem", 1, named_item)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::define_indexed_iterator(scope, prototype)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<HtmlAllCollectionStore>()
        .ok_or_else(|| "HTMLAllCollection state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create_for_document<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    document: v8::Local<'_, v8::Object>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let template = v8::ObjectTemplate::new(scope);
    unsafe {
        use v8::MapFnTo;

        let callback: v8::FunctionCallback = call_collection.map_fn_to();
        set_call_as_function_handler(&*template, callback, document.into());
        mark_as_undetectable(&*template);
    }
    let object = template
        .new_instance(scope)
        .ok_or_else(|| "cannot create callable HTMLAllCollection".to_owned())?;
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot set HTMLAllCollection prototype".to_owned());
    }
    let object_global = v8::Global::new(scope, object);
    let document_global = v8::Global::new(scope, document);
    scope
        .get_slot_mut::<HtmlAllCollectionStore>()
        .ok_or_else(|| "HTMLAllCollection state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            HtmlAllCollectionRecord {
                object: object_global,
                document: document_global,
                materialized_length: 0,
                named_keys: Vec::new(),
            },
        );
    refresh_one(scope, object);
    Ok(object)
}

pub(crate) fn refresh_all(scope: &mut v8::PinScope<'_, '_>) {
    let objects = scope
        .get_slot::<HtmlAllCollectionStore>()
        .map(|store| {
            store
                .records
                .values()
                .map(|record| record.object.clone())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    for object in objects {
        refresh_one(scope, v8::Local::new(scope, &object));
    }
}

fn refresh_one(scope: &mut v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) {
    let identity = object.get_identity_hash().get();
    let Some(snapshot) = scope
        .get_slot::<HtmlAllCollectionStore>()
        .and_then(|store| store.records.get(&identity))
        .cloned()
    else {
        return;
    };
    for index in 0..snapshot.materialized_length {
        let key: v8::Local<v8::Value> = v8::Integer::new_from_unsigned(scope, index as u32).into();
        let _ = object.delete(scope, key);
    }
    for name in snapshot.named_keys {
        if let Some(key) = v8::String::new(scope, &name) {
            let _ = object.delete(scope, key.into());
        }
    }
    let document = v8::Local::new(scope, &snapshot.document);
    let items = super::dom_selector::descendants(scope, document);
    for (index, item) in items.iter().enumerate() {
        let Some(key) = v8::String::new(scope, &index.to_string()) else {
            continue;
        };
        let _ = object.define_own_property(
            scope,
            key.into(),
            (*item).into(),
            v8::PropertyAttribute::READ_ONLY,
        );
    }
    let mut names = Vec::new();
    for item in &items {
        for attribute in ["id", "name"] {
            if attribute == "name" && !supports_name_attribute(scope, *item) {
                continue;
            }
            if let Some(value) = super::element::attribute_value(scope, *item, attribute)
                && !value.is_empty()
                && !names.contains(&value)
            {
                names.push(value);
            }
        }
    }
    for name in &names {
        let found = matching(scope, &items, name);
        let value: Option<v8::Local<v8::Value>> = match found.len() {
            0 => None,
            1 => Some(found[0].into()),
            _ => super::html_collection::create(scope, found)
                .ok()
                .map(Into::into),
        };
        if let (Some(key), Some(value)) = (v8::String::new(scope, name), value) {
            let _ = object.define_own_property(
                scope,
                key.into(),
                value,
                v8::PropertyAttribute::READ_ONLY,
            );
        }
    }
    if let Some(record) = scope
        .get_slot_mut::<HtmlAllCollectionStore>()
        .and_then(|store| store.records.get_mut(&identity))
    {
        record.materialized_length = items.len();
        record.named_keys = names;
    }
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor")
}

fn items_for_object<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<Vec<v8::Local<'s, v8::Object>>> {
    let record = scope
        .get_slot::<HtmlAllCollectionStore>()?
        .records
        .get(&object.get_identity_hash().get())?
        .clone();
    Some(super::dom_selector::descendants(
        scope,
        v8::Local::new(scope, &record.document),
    ))
}

fn items_for_document<'s>(
    scope: &v8::PinScope<'s, '_>,
    document: v8::Local<'_, v8::Object>,
) -> Vec<v8::Local<'s, v8::Object>> {
    super::dom_selector::descendants(scope, document)
}

fn get_length(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    refresh_one(scope, arguments.this());
    if let Some(items) = items_for_object(scope, arguments.this()) {
        result.set(v8::Integer::new_from_unsigned(scope, items.len() as u32).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}

fn matching<'s>(
    scope: &v8::PinScope<'s, '_>,
    items: &[v8::Local<'s, v8::Object>],
    name: &str,
) -> Vec<v8::Local<'s, v8::Object>> {
    let ids = items
        .iter()
        .copied()
        .filter(|item| super::element::attribute_value(scope, *item, "id").as_deref() == Some(name))
        .collect::<Vec<_>>();
    if !ids.is_empty() {
        return ids;
    }
    items
        .iter()
        .copied()
        .filter(|item| {
            supports_name_attribute(scope, *item)
                && super::element::attribute_value(scope, *item, "name").as_deref() == Some(name)
        })
        .collect()
}

fn supports_name_attribute(scope: &v8::PinScope<'_, '_>, item: v8::Local<'_, v8::Object>) -> bool {
    super::element::record(scope, item).is_some_and(|record| {
        matches!(
            record.tag_name.to_ascii_uppercase().as_str(),
            "EMBED" | "FORM" | "IFRAME" | "IMG" | "OBJECT"
        )
    })
}

fn return_named(
    scope: &mut v8::PinScope<'_, '_>,
    items: &[v8::Local<'_, v8::Object>],
    name: &str,
    mut result: v8::ReturnValue<'_>,
) {
    let found = matching(scope, items, name);
    match found.len() {
        0 => result.set(v8::null(scope).into()),
        1 => result.set(found[0].into()),
        _ => match super::html_collection::create(scope, found) {
            Ok(collection) => result.set(collection.into()),
            Err(message) => crate::webidl::throw_type_error(scope, &message),
        },
    }
}

fn return_item(
    scope: &mut v8::PinScope<'_, '_>,
    items: &[v8::Local<'_, v8::Object>],
    argument: v8::Local<'_, v8::Value>,
    mut result: v8::ReturnValue<'_>,
) {
    if argument.is_undefined() {
        result.set(v8::null(scope).into());
        return;
    }
    if argument.is_string() {
        let name = crate::webidl::value_to_string(scope, argument);
        if let Ok(index) = name.parse::<usize>() {
            if let Some(item) = items.get(index) {
                result.set((*item).into());
            } else {
                result.set(v8::null(scope).into());
            }
        } else {
            return_named(scope, items, &name, result);
        }
        return;
    }
    let index = argument.uint32_value(scope).unwrap_or(u32::MAX) as usize;
    if let Some(item) = items.get(index) {
        result.set((*item).into())
    } else {
        result.set(v8::null(scope).into())
    }
}

fn call_collection(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let Ok(document) = v8::Local::<v8::Object>::try_from(arguments.data()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let items = items_for_document(scope, document);
    return_item(scope, &items, arguments.get(0), result);
}

fn item(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    refresh_one(scope, arguments.this());
    let Some(items) = items_for_object(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    return_item(scope, &items, arguments.get(0), result);
}

fn named_item(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    refresh_one(scope, arguments.this());
    let Some(items) = items_for_object(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'namedItem' on 'HTMLAllCollection': 1 argument required, but only 0 present.",
        );
        return;
    }
    let name = crate::webidl::value_to_string(scope, arguments.get(0));
    return_named(scope, &items, &name, result)
}
