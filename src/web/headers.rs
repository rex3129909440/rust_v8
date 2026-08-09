use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct HeadersStore {
    constructors: HashMap<i32, v8::Global<v8::Function>>,
    records: HashMap<i32, Vec<(String, String)>>,
    iterator_prototypes: HashMap<i32, v8::Global<v8::Object>>,
    iterators: HashMap<i32, HeadersIteratorRecord>,
}

#[derive(Clone, Copy)]
enum HeadersIteratorKind {
    Entries,
    Keys,
    Values,
}

#[derive(Clone)]
struct HeadersIteratorRecord {
    headers_id: i32,
    index: usize,
    kind: HeadersIteratorKind,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(HeadersStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "Headers", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<HeadersStore>()
        .and_then(|store| store.constructors.get(&crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "Headers",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_method(scope, prototype, "append", 2, append)?;
    crate::webidl::define_method(scope, prototype, "delete", 1, delete)?;
    crate::webidl::define_method(scope, prototype, "get", 1, get)?;
    crate::webidl::define_method(scope, prototype, "getSetCookie", 0, get_set_cookie)?;
    crate::webidl::define_method(scope, prototype, "has", 1, has)?;
    crate::webidl::define_method(scope, prototype, "set", 2, set)?;
    crate::webidl::define_method(scope, prototype, "entries", 0, entries)?;
    crate::webidl::define_method(scope, prototype, "forEach", 1, for_each)?;
    crate::webidl::define_method(scope, prototype, "keys", 0, keys)?;
    crate::webidl::define_method(scope, prototype, "values", 0, values)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::define_to_string_tag(scope, prototype, "Headers")?;
    crate::webidl::define_iterator_alias(scope, prototype, "entries")?;
    let realm_id = crate::webidl::realm_id(scope);
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<HeadersStore>()
        .ok_or_else(|| "Headers state was not prepared".to_owned())?
        .constructors
        .insert(realm_id, stored);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    initial: Vec<(String, String)>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create Headers".to_owned());
    }
    let mut normalized = Vec::new();
    for (name, value) in initial {
        let name = normalize_name(&name)?;
        normalized.push((name, normalize_value(&value)?));
    }
    scope
        .get_slot_mut::<HeadersStore>()
        .ok_or_else(|| "Headers state was not prepared".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), normalized);
    Ok(object)
}

pub(crate) fn snapshot(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<Vec<(String, String)>> {
    scope
        .get_slot::<HeadersStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(scope, "Failed to construct 'Headers': use new");
        return;
    }
    let initial = match headers_init(scope, arguments.get(0)) {
        Ok(initial) => initial,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    scope
        .get_slot_mut::<HeadersStore>()
        .expect("Headers state")
        .records
        .insert(arguments.this().get_identity_hash().get(), initial);
    result.set(arguments.this().into());
}

fn headers_init(
    scope: &mut v8::PinScope<'_, '_>,
    init: v8::Local<'_, v8::Value>,
) -> Result<Vec<(String, String)>, String> {
    if init.is_undefined() {
        return Ok(Vec::new());
    }
    let object = v8::Local::<v8::Object>::try_from(init).map_err(|_| {
        "The provided value is not of type '(record<ByteString, ByteString> or sequence<sequence<ByteString>>)'."
            .to_owned()
    })?;
    if let Some(existing) = snapshot(scope, object) {
        return Ok(existing);
    }

    let iterator_key = v8::Symbol::get_iterator(scope);
    let iterator_method = object
        .get(scope, iterator_key.into())
        .ok_or_else(|| "Cannot read Headers initializer iterator".to_owned())?;
    if !iterator_method.is_undefined() && !iterator_method.is_null() {
        if v8::Local::<v8::Function>::try_from(iterator_method).is_err() {
            return Err("The object must have a callable @@iterator property.".to_owned());
        }
        let outer = crate::webidl::sequence_values(scope, init)?;
        let mut initial = Vec::with_capacity(outer.len());
        for pair in outer {
            let pair = v8::Local::new(scope, &pair);
            let pair = crate::webidl::sequence_values(scope, pair)?;
            if pair.len() != 2 {
                return Err("Invalid value".to_owned());
            }
            let name = byte_string(scope, v8::Local::new(scope, &pair[0]))?;
            let value = byte_string(scope, v8::Local::new(scope, &pair[1]))?;
            initial.push((normalize_name(&name)?, normalize_value(&value)?));
        }
        return Ok(initial);
    }

    let names = object
        .get_own_property_names(
            scope,
            v8::GetPropertyNamesArgs {
                mode: v8::KeyCollectionMode::OwnOnly,
                property_filter: v8::PropertyFilter::ONLY_ENUMERABLE,
                index_filter: v8::IndexFilter::IncludeIndices,
                key_conversion: v8::KeyConversionMode::ConvertToString,
            },
        )
        .ok_or_else(|| "Cannot enumerate Headers record".to_owned())?;
    let mut initial = Vec::with_capacity(names.length() as usize);
    for index in 0..names.length() {
        let key = names
            .get_index(scope, index)
            .ok_or_else(|| "Cannot read Headers record key".to_owned())?;
        let value = object
            .get(scope, key)
            .ok_or_else(|| "Cannot read Headers record value".to_owned())?;
        let name = byte_string(scope, key)?;
        let value = byte_string(scope, value)?;
        initial.push((normalize_name(&name)?, normalize_value(&value)?));
    }
    Ok(initial)
}

fn byte_string(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> Result<String, String> {
    let value = value
        .to_string(scope)
        .ok_or_else(|| "Cannot convert value to a ByteString".to_owned())?
        .to_rust_string_lossy(scope);
    if value.chars().any(|character| u32::from(character) > 0xff) {
        Err("String contains non ISO-8859-1 code point.".to_owned())
    } else {
        Ok(value)
    }
}

fn normalize_name(name: &str) -> Result<String, String> {
    let name = name.to_ascii_lowercase();
    if name.is_empty()
        || name
            .bytes()
            .any(|byte| !byte.is_ascii_alphanumeric() && !b"!#$%&'*+-.^_`|~".contains(&byte))
    {
        return Err("Invalid name".to_owned());
    }
    Ok(name)
}

fn normalize_value(value: &str) -> Result<String, String> {
    let value = value.trim_matches(|character: char| character == ' ' || character == '\t');
    if value
        .bytes()
        .any(|byte| matches!(byte, b'\0' | b'\r' | b'\n'))
    {
        Err("Invalid value".to_owned())
    } else {
        Ok(value.to_owned())
    }
}

fn values_for(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<Vec<(String, String)>> {
    snapshot(scope, object)
}

fn update(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut Vec<(String, String)>),
) -> bool {
    if let Some(values) = scope
        .get_slot_mut::<HeadersStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        change(values);
        true
    } else {
        false
    }
}

fn header_name(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> Option<String> {
    let value = match byte_string(scope, value) {
        Ok(value) => value,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return None;
        }
    };
    match normalize_name(&value) {
        Ok(value) => Some(value),
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            None
        }
    }
}

fn header_value(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> Option<String> {
    let value = match byte_string(scope, value).and_then(|value| normalize_value(&value)) {
        Ok(value) => value,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return None;
        }
    };
    Some(value)
}

fn required(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: &v8::FunctionCallbackArguments<'_>,
    count: i32,
    method: &str,
) -> bool {
    if arguments.length() >= count {
        true
    } else {
        crate::webidl::throw_type_error(
            scope,
            &format!(
                "Failed to execute '{method}' on 'Headers': {count} arguments required, but only {} present.",
                arguments.length()
            ),
        );
        false
    }
}
fn return_string(scope: &mut v8::PinScope<'_, '_>, value: &str, mut result: v8::ReturnValue<'_>) {
    if let Some(value) = v8::String::new(scope, value) {
        result.set(value.into())
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
    let Some(name) = header_name(scope, arguments.get(0)) else {
        return;
    };
    let Some(value) = header_value(scope, arguments.get(1)) else {
        return;
    };
    if !update(scope, arguments.this(), |values| values.push((name, value))) {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
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
    let Some(name) = header_name(scope, arguments.get(0)) else {
        return;
    };
    if !update(scope, arguments.this(), |values| {
        values.retain(|(current, _)| current != &name)
    }) {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
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
    let Some(name) = header_name(scope, arguments.get(0)) else {
        return;
    };
    let Some(values) = values_for(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let matched: Vec<&str> = values
        .iter()
        .filter(|(current, _)| current == &name)
        .map(|(_, value)| value.as_str())
        .collect();
    if matched.is_empty() {
        result.set(v8::null(scope).into())
    } else {
        return_string(scope, &matched.join(", "), result)
    }
}
fn get_set_cookie(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(values) = values_for(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let matched: Vec<&str> = values
        .iter()
        .filter(|(name, _)| name == "set-cookie")
        .map(|(_, value)| value.as_str())
        .collect();
    let array = v8::Array::new(scope, matched.len() as i32);
    for (index, value) in matched.iter().enumerate() {
        if let Some(value) = v8::String::new(scope, value) {
            let _ = array.set_index(scope, index as u32, value.into());
        }
    }
    result.set(array.into())
}
fn has(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !required(scope, &arguments, 1, "has") {
        return;
    }
    let Some(name) = header_name(scope, arguments.get(0)) else {
        return;
    };
    if let Some(values) = values_for(scope, arguments.this()) {
        result
            .set(v8::Boolean::new(scope, values.iter().any(|(current, _)| current == &name)).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
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
    let Some(name) = header_name(scope, arguments.get(0)) else {
        return;
    };
    let Some(value) = header_value(scope, arguments.get(1)) else {
        return;
    };
    if !update(scope, arguments.this(), |values| {
        values.retain(|(current, _)| current != &name);
        values.push((name, value))
    }) {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}

fn combined(values: &[(String, String)]) -> Vec<(String, String)> {
    let mut output: Vec<(String, String)> = Vec::new();
    for (name, value) in values {
        if name == "set-cookie" {
            output.push((name.clone(), value.clone()));
            continue;
        }
        if let Some((_, existing)) = output.iter_mut().find(|(current, _)| current == name) {
            existing.push_str(", ");
            existing.push_str(value)
        } else {
            output.push((name.clone(), value.clone()))
        }
    }
    output.sort_by(|a, b| a.0.cmp(&b.0));
    output
}

fn ensure_iterator_prototype<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(prototype) = scope
        .get_slot::<HeadersStore>()
        .and_then(|store| store.iterator_prototypes.get(&realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &prototype));
    }
    let prototype = v8::Object::new(scope);
    let array = v8::Array::new(scope, 0);
    let values_key = crate::webidl::string(scope, "values")?;
    let values = array
        .get(scope, values_key.into())
        .and_then(|value| v8::Local::<v8::Function>::try_from(value).ok())
        .ok_or_else(|| "Array.prototype.values is unavailable".to_owned())?;
    let array_iterator = values
        .call(scope, array.into(), &[])
        .and_then(|value| v8::Local::<v8::Object>::try_from(value).ok())
        .ok_or_else(|| "cannot create intrinsic Array Iterator".to_owned())?;
    let array_iterator_prototype = array_iterator
        .get_prototype(scope)
        .and_then(|value| v8::Local::<v8::Object>::try_from(value).ok())
        .ok_or_else(|| "Array Iterator prototype is unavailable".to_owned())?;
    let iterator_prototype = array_iterator_prototype
        .get_prototype(scope)
        .ok_or_else(|| "Iterator prototype is unavailable".to_owned())?;
    if prototype.set_prototype(scope, iterator_prototype) != Some(true) {
        return Err("cannot inherit Headers Iterator prototype".to_owned());
    }
    crate::trace::label_native_value_once(scope, prototype.into(), "Headers Iterator.prototype");
    crate::webidl::define_method(scope, prototype, "next", 0, iterator_next)?;
    crate::webidl::define_to_string_tag(scope, prototype, "Headers Iterator")?;
    let stored = v8::Global::new(scope, prototype);
    scope
        .get_slot_mut::<HeadersStore>()
        .ok_or_else(|| "Headers state was not prepared".to_owned())?
        .iterator_prototypes
        .insert(realm_id, stored);
    Ok(prototype)
}

fn create_iterator<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    headers_id: i32,
    kind: HeadersIteratorKind,
) -> Result<v8::Local<'s, v8::Object>, String> {
    if !scope
        .get_slot::<HeadersStore>()
        .is_some_and(|store| store.records.contains_key(&headers_id))
    {
        return Err("Illegal invocation".to_owned());
    }
    let prototype = ensure_iterator_prototype(scope)?;
    let iterator = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, iterator, prototype.into()) != Some(true) {
        return Err("cannot create Headers Iterator".to_owned());
    }
    scope
        .get_slot_mut::<HeadersStore>()
        .ok_or_else(|| "Headers state was not prepared".to_owned())?
        .iterators
        .insert(
            iterator.get_identity_hash().get(),
            HeadersIteratorRecord {
                headers_id,
                index: 0,
                kind,
            },
        );
    Ok(iterator)
}

fn iterator_result<'s>(
    scope: &v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Value>,
    done: bool,
) -> v8::Local<'s, v8::Object> {
    let result = v8::Object::new(scope);
    if let Some(key) = v8::String::new(scope, "value") {
        let _ = result.create_data_property(scope, key.into(), value);
    }
    if let Some(key) = v8::String::new(scope, "done") {
        let _ =
            result.create_data_property(scope, key.into(), v8::Boolean::new(scope, done).into());
    }
    result
}

fn iterator_next(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let iterator_id = arguments.this().get_identity_hash().get();
    let Some(iterator) = scope
        .get_slot::<HeadersStore>()
        .and_then(|store| store.iterators.get(&iterator_id))
        .cloned()
    else {
        crate::webidl::throw_type_error(
            scope,
            "Method Headers Iterator.prototype.next called on incompatible receiver",
        );
        return;
    };
    let item = scope
        .get_slot::<HeadersStore>()
        .and_then(|store| store.records.get(&iterator.headers_id))
        .map(|values| combined(values))
        .and_then(|values| values.get(iterator.index).cloned());
    let Some((name, value)) = item else {
        let undefined = v8::undefined(scope);
        result.set(iterator_result(scope, undefined.into(), true).into());
        return;
    };
    if let Some(record) = scope
        .get_slot_mut::<HeadersStore>()
        .and_then(|store| store.iterators.get_mut(&iterator_id))
    {
        record.index += 1;
    }
    let Some(name) = v8::String::new(scope, &name) else {
        return;
    };
    let Some(value) = v8::String::new(scope, &value) else {
        return;
    };
    let output: v8::Local<'_, v8::Value> = match iterator.kind {
        HeadersIteratorKind::Entries => {
            let pair = v8::Array::new(scope, 2);
            let _ = pair.set_index(scope, 0, name.into());
            let _ = pair.set_index(scope, 1, value.into());
            pair.into()
        }
        HeadersIteratorKind::Keys => name.into(),
        HeadersIteratorKind::Values => value.into(),
    };
    result.set(iterator_result(scope, output, false).into());
}

fn return_iterator(
    scope: &mut v8::PinScope<'_, '_>,
    receiver: v8::Local<'_, v8::Object>,
    kind: HeadersIteratorKind,
    mut result: v8::ReturnValue<'_>,
) {
    let headers_id = receiver.get_identity_hash().get();
    match create_iterator(scope, headers_id, kind) {
        Ok(iterator) => result.set(iterator.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
fn entries(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    return_iterator(
        scope,
        arguments.this(),
        HeadersIteratorKind::Entries,
        result,
    )
}
fn keys(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    return_iterator(scope, arguments.this(), HeadersIteratorKind::Keys, result)
}
fn values(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    return_iterator(scope, arguments.this(), HeadersIteratorKind::Values, result)
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
        crate::webidl::throw_type_error(scope, "callback must be a function");
        return;
    };
    let headers_id = arguments.this().get_identity_hash().get();
    if !scope
        .get_slot::<HeadersStore>()
        .is_some_and(|store| store.records.contains_key(&headers_id))
    {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let receiver = arguments.get(1);
    let mut index = 0;
    loop {
        let item = scope
            .get_slot::<HeadersStore>()
            .and_then(|store| store.records.get(&headers_id))
            .map(|values| combined(values))
            .and_then(|values| values.get(index).cloned());
        let Some((name, value)) = item else {
            break;
        };
        index += 1;
        let Some(name) = v8::String::new(scope, &name) else {
            return;
        };
        let Some(value) = v8::String::new(scope, &value) else {
            return;
        };
        if callback
            .call(
                scope,
                receiver,
                &[value.into(), name.into(), arguments.this().into()],
            )
            .is_none()
        {
            return;
        }
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<HeadersStore>() {
        store.constructors.remove(&realm_id);
        store.iterator_prototypes.remove(&realm_id);
    }
}
