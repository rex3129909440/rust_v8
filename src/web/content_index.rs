use std::collections::{HashMap, HashSet};

#[derive(Clone)]
struct Description {
    id: String,
    title: String,
    description: String,
    category: String,
}

#[derive(Default)]
pub(crate) struct ContentIndexStore {
    constructor: crate::webidl::RealmConstructor,
    instances: HashSet<i32>,
    entries: HashMap<i32, Vec<Description>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(ContentIndexStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure(scope)?;
    crate::webidl::define_global(scope, "ContentIndex", constructor.into())
}

fn ensure<'s>(scope: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm = crate::webidl::realm_id(scope);
    if let Some(value) = scope
        .get_slot::<ContentIndexStore>()
        .and_then(|s| s.constructor.get(realm))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &value));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "ContentIndex",
        0,
        v8::ConstructorBehavior::Allow,
        super::android_api_support::illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_method(scope, prototype, "add", 1, add)?;
    crate::webidl::define_method(scope, prototype, "delete", 1, delete)?;
    crate::webidl::define_method(scope, prototype, "getAll", 0, get_all)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    super::android_api_support::set_tag(scope, prototype, "ContentIndex")?;
    let stored_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<ContentIndexStore>()
        .unwrap()
        .constructor
        .insert(realm, stored_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create ContentIndex".to_owned());
    }
    let id = object.get_identity_hash().get();
    let store = scope.get_slot_mut::<ContentIndexStore>().unwrap();
    store.instances.insert(id);
    store.entries.insert(id, Vec::new());
    Ok(object)
}

fn valid(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: &v8::FunctionCallbackArguments<'_>,
    operation: &str,
) -> Option<i32> {
    let id = arguments.this().get_identity_hash().get();
    let valid = scope
        .get_slot::<ContentIndexStore>()
        .expect("ContentIndex state")
        .instances
        .contains(&id);
    super::android_api_support::require_brand(scope, valid, "ContentIndex", operation).then_some(id)
}

fn add(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(id) = valid(scope, &arguments, "add") else {
        return;
    };
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'add' on 'ContentIndex': 1 argument required, but only 0 present.",
        );
        return;
    }
    let Ok(value) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'add' on 'ContentIndex': parameter 1 is not of type 'ContentDescription'.",
        );
        return;
    };
    let entry = Description {
        id: super::android_api_support::string_property(scope, value, "id"),
        title: super::android_api_support::string_property(scope, value, "title"),
        description: super::android_api_support::string_property(scope, value, "description"),
        category: super::android_api_support::string_property(scope, value, "category"),
    };
    if entry.id.is_empty()
        || entry.title.is_empty()
        || entry.description.is_empty()
        || entry.category.is_empty()
    {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'add' on 'ContentIndex': required ContentDescription member is undefined.",
        );
        return;
    }
    let entries = scope
        .get_slot_mut::<ContentIndexStore>()
        .unwrap()
        .entries
        .entry(id)
        .or_default();
    entries.retain(|existing| existing.id != entry.id);
    entries.push(entry);
    if let Some(promise) = super::android_api_support::resolved_undefined(scope) {
        result.set(promise.into());
    }
}

fn delete(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(id) = valid(scope, &arguments, "delete") else {
        return;
    };
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'delete' on 'ContentIndex': 1 argument required, but only 0 present.",
        );
        return;
    }
    let target = crate::webidl::value_to_string(scope, arguments.get(0));
    if let Some(entries) = scope
        .get_slot_mut::<ContentIndexStore>()
        .unwrap()
        .entries
        .get_mut(&id)
    {
        entries.retain(|entry| entry.id != target);
    }
    if let Some(promise) = super::android_api_support::resolved_undefined(scope) {
        result.set(promise.into());
    }
}

fn get_all(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(id) = valid(scope, &arguments, "getAll") else {
        return;
    };
    let entries = scope
        .get_slot::<ContentIndexStore>()
        .and_then(|store| store.entries.get(&id))
        .cloned()
        .unwrap_or_default();
    let values = v8::Array::new(scope, entries.len() as i32);
    for (index, entry) in entries.iter().enumerate() {
        let object = v8::Object::new(scope);
        for (name, value) in [
            ("id", &entry.id),
            ("title", &entry.title),
            ("description", &entry.description),
            ("category", &entry.category),
        ] {
            if let (Some(key), Some(value)) =
                (v8::String::new(scope, name), v8::String::new(scope, value))
            {
                let _ = object.set(scope, key.into(), value.into());
            }
        }
        let _ = values.set_index(scope, index as u32, object.into());
    }
    if let Ok(promise) = super::writable_stream::resolved_promise(scope, values.into()) {
        result.set(promise.into());
    }
}
