use std::collections::HashMap;

#[derive(Clone)]
struct FormEntry {
    name: String,
    value: v8::Global<v8::Value>,
}

#[derive(Default)]
pub(crate) struct FormDataStore {
    constructors: HashMap<i32, v8::Global<v8::Function>>,
    records: HashMap<i32, Vec<FormEntry>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(FormDataStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "FormData", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<FormDataStore>()
        .and_then(|store| store.constructors.get(&crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "FormData",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_method(scope, prototype, "append", 2, append)?;
    crate::webidl::define_method(scope, prototype, "delete", 1, delete)?;
    crate::webidl::define_method(scope, prototype, "get", 1, get)?;
    crate::webidl::define_method(scope, prototype, "getAll", 1, get_all)?;
    crate::webidl::define_method(scope, prototype, "has", 1, has)?;
    crate::webidl::define_method(scope, prototype, "set", 2, set)?;
    crate::webidl::define_method(scope, prototype, "entries", 0, entries)?;
    crate::webidl::define_method(scope, prototype, "forEach", 1, for_each)?;
    crate::webidl::define_method(scope, prototype, "keys", 0, keys)?;
    crate::webidl::define_method(scope, prototype, "values", 0, values)?;
    let entries_key = crate::webidl::string(scope, "entries")?;
    let entries_function = prototype
        .get(scope, entries_key.into())
        .ok_or_else(|| "cannot read FormData entries".to_owned())?;
    let iterator_symbol = v8::Symbol::get_iterator(scope);
    if prototype.define_own_property(
        scope,
        iterator_symbol.into(),
        entries_function,
        v8::PropertyAttribute::DONT_ENUM,
    ) != Some(true)
    {
        return Err("cannot define FormData iterator".to_owned());
    }
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::move_iterator_to_end(scope, prototype)?;
    let realm_id = crate::webidl::realm_id(scope);
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<FormDataStore>()
        .ok_or_else(|| "FormData state was not prepared".to_owned())?
        .constructors
        .insert(realm_id, stored);
    Ok(constructor)
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'FormData': Please use the 'new' operator",
        );
        return;
    }
    if !arguments.get(0).is_undefined() && !arguments.get(0).is_null() {
        let Ok(form) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
            crate::webidl::throw_type_error(scope, "parameter 1 is not of type 'HTMLFormElement'");
            return;
        };
        if !super::html_form_element::is_form(scope, form) {
            crate::webidl::throw_type_error(scope, "parameter 1 is not of type 'HTMLFormElement'");
            return;
        }
    }
    scope
        .get_slot_mut::<FormDataStore>()
        .expect("FormData state")
        .records
        .insert(arguments.this().get_identity_hash().get(), Vec::new());
    result.set(arguments.this().into());
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create FormData".to_owned());
    }
    scope
        .get_slot_mut::<FormDataStore>()
        .ok_or_else(|| "FormData state was not prepared".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), Vec::new());
    Ok(object)
}

pub(crate) fn is_form_data(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> bool {
    scope.get_slot::<FormDataStore>().is_some_and(|store| {
        store
            .records
            .contains_key(&object.get_identity_hash().get())
    })
}

fn snapshot(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<Vec<FormEntry>> {
    scope
        .get_slot::<FormDataStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn entry_value(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> v8::Global<v8::Value> {
    if value.is_object() {
        v8::Global::new(scope, value)
    } else {
        let string = crate::webidl::value_to_string(scope, value);
        let value: v8::Local<'_, v8::Value> = v8::String::new(scope, &string)
            .map(|value| value.into())
            .unwrap_or_else(|| v8::undefined(scope).into());
        v8::Global::new(scope, value)
    }
}

fn required(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: &v8::FunctionCallbackArguments<'_>,
    count: i32,
    method: &str,
) -> bool {
    if arguments.length() < count {
        crate::webidl::throw_type_error(
            scope,
            &format!("Failed to execute '{method}' on 'FormData': {count} arguments required"),
        );
        false
    } else {
        true
    }
}

fn append(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if !required(scope, &arguments, 2, "append") {
        return;
    }
    let name = crate::webidl::value_to_string(scope, arguments.get(0));
    let value = entry_value(scope, arguments.get(1));
    if let Some(entries) = scope.get_slot_mut::<FormDataStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) {
        entries.push(FormEntry { name, value });
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn delete(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if !required(scope, &arguments, 1, "delete") {
        return;
    }
    let name = crate::webidl::value_to_string(scope, arguments.get(0));
    if let Some(entries) = scope.get_slot_mut::<FormDataStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) {
        entries.retain(|entry| entry.name != name);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !required(scope, &arguments, 1, "get") {
        return;
    }
    let name = crate::webidl::value_to_string(scope, arguments.get(0));
    let Some(entries) = snapshot(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(entry) = entries.iter().find(|entry| entry.name == name) {
        result.set(v8::Local::new(scope, &entry.value));
    } else {
        result.set(v8::null(scope).into());
    }
}

fn get_all(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !required(scope, &arguments, 1, "getAll") {
        return;
    }
    let name = crate::webidl::value_to_string(scope, arguments.get(0));
    let Some(entries) = snapshot(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let matched: Vec<FormEntry> = entries
        .into_iter()
        .filter(|entry| entry.name == name)
        .collect();
    let array = v8::Array::new(scope, matched.len() as i32);
    for (index, entry) in matched.iter().enumerate() {
        let value = v8::Local::new(scope, &entry.value);
        let _ = array.set_index(scope, index as u32, value);
    }
    result.set(array.into());
}

fn has(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !required(scope, &arguments, 1, "has") {
        return;
    }
    let name = crate::webidl::value_to_string(scope, arguments.get(0));
    if let Some(entries) = snapshot(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, entries.iter().any(|entry| entry.name == name)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if !required(scope, &arguments, 2, "set") {
        return;
    }
    let name = crate::webidl::value_to_string(scope, arguments.get(0));
    let value = entry_value(scope, arguments.get(1));
    let Some(entries) = scope.get_slot_mut::<FormDataStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(first) = entries.iter().position(|entry| entry.name == name) {
        entries[first].value = value;
        let mut index = entries.len();
        while index > first + 1 {
            index -= 1;
            if entries[index].name == name {
                entries.remove(index);
            }
        }
    } else {
        entries.push(FormEntry { name, value });
    }
}

fn return_iterator(
    scope: &mut v8::PinScope<'_, '_>,
    array: v8::Local<'_, v8::Array>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(key) = v8::String::new(scope, "values") else {
        return;
    };
    let Some(method) = array
        .get(scope, key.into())
        .and_then(|value| v8::Local::<v8::Function>::try_from(value).ok())
    else {
        return;
    };
    if let Some(iterator) = method.call(scope, array.into(), &[]) {
        result.set(iterator);
    }
}

fn entries(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let Some(entries) = snapshot(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let array = v8::Array::new(scope, entries.len() as i32);
    for (index, entry) in entries.iter().enumerate() {
        let pair = v8::Array::new(scope, 2);
        if let Some(name) = v8::String::new(scope, &entry.name) {
            let _ = pair.set_index(scope, 0, name.into());
        }
        let value = v8::Local::new(scope, &entry.value);
        let _ = pair.set_index(scope, 1, value);
        let _ = array.set_index(scope, index as u32, pair.into());
    }
    return_iterator(scope, array, result);
}

fn keys(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let Some(entries) = snapshot(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let array = v8::Array::new(scope, entries.len() as i32);
    for (index, entry) in entries.iter().enumerate() {
        if let Some(name) = v8::String::new(scope, &entry.name) {
            let _ = array.set_index(scope, index as u32, name.into());
        }
    }
    return_iterator(scope, array, result);
}

fn values(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let Some(entries) = snapshot(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let array = v8::Array::new(scope, entries.len() as i32);
    for (index, entry) in entries.iter().enumerate() {
        let value = v8::Local::new(scope, &entry.value);
        let _ = array.set_index(scope, index as u32, value);
    }
    return_iterator(scope, array, result);
}

fn for_each(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if !required(scope, &arguments, 1, "forEach") {
        return;
    }
    let Ok(callback) = v8::Local::<v8::Function>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "The callback must be a function");
        return;
    };
    let Some(entries) = snapshot(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let this_value = arguments.get(1);
    for entry in entries {
        let value = v8::Local::new(scope, &entry.value);
        let Some(name) = v8::String::new(scope, &entry.name) else {
            continue;
        };
        let _ = callback.call(
            scope,
            this_value,
            &[value, name.into(), arguments.this().into()],
        );
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<FormDataStore>() {
        store.constructors.remove(&realm_id);
    }
}
