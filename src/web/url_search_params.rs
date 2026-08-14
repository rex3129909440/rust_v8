use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct UrlSearchParamsStore {
    constructors: HashMap<i32, v8::Global<v8::Function>>,
    records: HashMap<i32, ParamsRecord>,
    iterator_prototypes: HashMap<i32, v8::Global<v8::Object>>,
    iterators: HashMap<i32, ParamsIteratorRecord>,
}

#[derive(Clone)]
struct ParamsRecord {
    pairs: Vec<(String, String)>,
    owner_url: Option<i32>,
}

#[derive(Clone, Copy)]
enum ParamsIteratorKind {
    Entries,
    Keys,
    Values,
}

#[derive(Clone)]
struct ParamsIteratorRecord {
    params_id: i32,
    index: usize,
    kind: ParamsIteratorKind,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(UrlSearchParamsStore::default());
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<UrlSearchParamsStore>()
        .and_then(|store| store.constructors.get(&crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }

    let constructor = crate::webidl::create_function(
        scope,
        "URLSearchParams",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "size", get_size)?;
    crate::webidl::define_method(scope, prototype, "append", 2, append)?;
    crate::webidl::define_method(scope, prototype, "delete", 1, delete)?;
    crate::webidl::define_method(scope, prototype, "get", 1, get)?;
    crate::webidl::define_method(scope, prototype, "getAll", 1, get_all)?;
    crate::webidl::define_method(scope, prototype, "has", 1, has)?;
    crate::webidl::define_method(scope, prototype, "set", 2, set)?;
    crate::webidl::define_method(scope, prototype, "sort", 0, sort)?;
    crate::webidl::define_method(scope, prototype, "toString", 0, to_string)?;
    crate::webidl::define_method(scope, prototype, "entries", 0, entries)?;
    crate::webidl::define_method(scope, prototype, "forEach", 1, for_each)?;
    crate::webidl::define_method(scope, prototype, "keys", 0, keys)?;
    crate::webidl::define_method(scope, prototype, "values", 0, values)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::define_to_string_tag(scope, prototype, "URLSearchParams")?;

    let entries_name = crate::webidl::string(scope, "entries")?;
    let entries_function = prototype
        .get(scope, entries_name.into())
        .ok_or_else(|| "URLSearchParams.entries is missing".to_owned())?;
    let iterator_symbol = v8::Symbol::get_iterator(scope);
    if prototype.define_own_property(
        scope,
        iterator_symbol.into(),
        entries_function,
        v8::PropertyAttribute::DONT_ENUM,
    ) != Some(true)
    {
        return Err("cannot define URLSearchParams iterator".to_owned());
    }

    let realm_id = crate::webidl::realm_id(scope);
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<UrlSearchParamsStore>()
        .ok_or_else(|| "URLSearchParams state was not prepared".to_owned())?
        .constructors
        .insert(realm_id, stored);
    Ok(constructor)
}

#[allow(dead_code)]
pub(crate) fn install_global(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "URLSearchParams", constructor.into())
}

pub(crate) fn create_linked<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    query: &str,
    owner_url: i32,
) -> Result<(v8::Local<'s, v8::Object>, i32), String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create URLSearchParams object".to_owned());
    }
    let id = object.get_identity_hash().get();
    let pairs = parse_query(query);
    scope
        .get_slot_mut::<UrlSearchParamsStore>()
        .ok_or_else(|| "URLSearchParams state is missing".to_owned())?
        .records
        .insert(
            id,
            ParamsRecord {
                pairs,
                owner_url: Some(owner_url),
            },
        );
    Ok((object, id))
}

pub(crate) fn replace_query(scope: &mut v8::PinScope<'_, '_>, params_id: i32, query: &str) {
    if let Some(record) = scope
        .get_slot_mut::<UrlSearchParamsStore>()
        .and_then(|store| store.records.get_mut(&params_id))
    {
        record.pairs = parse_query(query);
    }
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'URLSearchParams': Please use the 'new' operator",
        );
        return;
    }

    let pairs = match pairs_from_value(scope, arguments.get(0)) {
        Ok(pairs) => pairs,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    let object = arguments.this();
    let id = object.get_identity_hash().get();
    scope
        .get_slot_mut::<UrlSearchParamsStore>()
        .expect("URLSearchParams state")
        .records
        .insert(
            id,
            ParamsRecord {
                pairs,
                owner_url: None,
            },
        );
    result.set(object.into());
}

fn pairs_from_value(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> Result<Vec<(String, String)>, String> {
    if value.is_undefined() {
        return Ok(Vec::new());
    }
    if value.is_string() || value.is_string_object() {
        return Ok(parse_query(&crate::webidl::value_to_string(scope, value)));
    }
    let Ok(object) = v8::Local::<v8::Object>::try_from(value) else {
        return Ok(parse_query(&crate::webidl::value_to_string(scope, value)));
    };

    let iterator_key = v8::Symbol::get_iterator(scope);
    let iterator_method = object
        .get(scope, iterator_key.into())
        .ok_or_else(|| "Cannot read URLSearchParams initializer iterator".to_owned())?;
    if !iterator_method.is_undefined() && !iterator_method.is_null() {
        if v8::Local::<v8::Function>::try_from(iterator_method).is_err() {
            return Err("The object must have a callable @@iterator property.".to_owned());
        }
        let outer = crate::webidl::sequence_values(scope, value)?;
        let mut pairs = Vec::with_capacity(outer.len());
        for entry in outer {
            let entry = v8::Local::new(scope, &entry);
            let inner = crate::webidl::sequence_values(scope, entry)?;
            if inner.len() != 2 {
                return Err("Sequence initializer must only contain pair elements".to_owned());
            }
            let key = v8::Local::new(scope, &inner[0]);
            let value = v8::Local::new(scope, &inner[1]);
            pairs.push((
                crate::webidl::value_to_string(scope, key),
                crate::webidl::value_to_string(scope, value),
            ));
        }
        return Ok(pairs);
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
        .ok_or_else(|| "Cannot enumerate URLSearchParams record".to_owned())?;
    let mut pairs = Vec::with_capacity(names.length() as usize);
    for index in 0..names.length() {
        let key = names
            .get_index(scope, index)
            .ok_or_else(|| "Cannot read URLSearchParams record key".to_owned())?;
        let value = object
            .get(scope, key)
            .ok_or_else(|| "Cannot read URLSearchParams record value".to_owned())?;
        pairs.push((
            crate::webidl::value_to_string(scope, key),
            crate::webidl::value_to_string(scope, value),
        ));
    }
    Ok(pairs)
}

fn parse_query(query: &str) -> Vec<(String, String)> {
    let query = query.strip_prefix('?').unwrap_or(query);
    url::form_urlencoded::parse(query.as_bytes())
        .map(|(name, value)| (name.into_owned(), value.into_owned()))
        .collect()
}

fn serialize(pairs: &[(String, String)]) -> String {
    let mut serializer = url::form_urlencoded::Serializer::new(String::new());
    for (name, value) in pairs {
        serializer.append_pair(name, value);
    }
    serializer.finish()
}

fn record(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> Option<ParamsRecord> {
    scope
        .get_slot::<UrlSearchParamsStore>()
        .and_then(|store| store.records.get(&object.get_identity_hash().get()))
        .cloned()
}

pub(crate) fn serialized_snapshot(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<String> {
    record(scope, object).map(|record| serialize(&record.pairs))
}

fn with_record_mut(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    operation: impl FnOnce(&mut ParamsRecord),
) {
    let id = object.get_identity_hash().get();
    let owner = if let Some(record) = scope
        .get_slot_mut::<UrlSearchParamsStore>()
        .and_then(|store| store.records.get_mut(&id))
    {
        operation(record);
        record.owner_url
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(owner) = owner {
        let query = scope
            .get_slot::<UrlSearchParamsStore>()
            .and_then(|store| store.records.get(&id))
            .map(|record| serialize(&record.pairs))
            .unwrap_or_default();
        super::url::set_query_from_params(scope, owner, &query);
    }
}

fn require_argument_count(
    scope: &v8::PinScope<'_, '_>,
    actual: i32,
    expected: i32,
    method: &str,
) -> bool {
    if actual >= expected {
        true
    } else {
        crate::webidl::throw_type_error(
            scope,
            &format!(
                "Failed to execute '{method}' on 'URLSearchParams': {expected} arguments required"
            ),
        );
        false
    }
}

fn get_size(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new_from_unsigned(scope, record.pairs.len() as u32).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn append(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if !require_argument_count(scope, arguments.length(), 2, "append") {
        return;
    }
    let name = crate::webidl::value_to_string(scope, arguments.get(0));
    let value = crate::webidl::value_to_string(scope, arguments.get(1));
    with_record_mut(scope, arguments.this(), |record| {
        record.pairs.push((name, value));
    });
}

fn delete(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if !require_argument_count(scope, arguments.length(), 1, "delete") {
        return;
    }
    let name = crate::webidl::value_to_string(scope, arguments.get(0));
    let value =
        (arguments.length() > 1).then(|| crate::webidl::value_to_string(scope, arguments.get(1)));
    with_record_mut(scope, arguments.this(), |record| {
        record.pairs.retain(|(pair_name, pair_value)| {
            pair_name != &name || value.as_ref().is_some_and(|value| pair_value != value)
        });
    });
}

fn get(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !require_argument_count(scope, arguments.length(), 1, "get") {
        return;
    }
    let name = crate::webidl::value_to_string(scope, arguments.get(0));
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some((_, value)) = record
        .pairs
        .iter()
        .find(|(pair_name, _)| pair_name == &name)
    {
        if let Some(value) = v8::String::new(scope, value) {
            result.set(value.into());
        }
    } else {
        result.set(v8::null(scope).into());
    }
}

fn get_all(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !require_argument_count(scope, arguments.length(), 1, "getAll") {
        return;
    }
    let name = crate::webidl::value_to_string(scope, arguments.get(0));
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let values: Vec<&str> = record
        .pairs
        .iter()
        .filter(|(pair_name, _)| pair_name == &name)
        .map(|(_, value)| value.as_str())
        .collect();
    let array = v8::Array::new(scope, values.len() as i32);
    for (index, value) in values.into_iter().enumerate() {
        if let Some(value) = v8::String::new(scope, value) {
            let _ = array.set_index(scope, index as u32, value.into());
        }
    }
    result.set(array.into());
}

fn has(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !require_argument_count(scope, arguments.length(), 1, "has") {
        return;
    }
    let name = crate::webidl::value_to_string(scope, arguments.get(0));
    let value =
        (arguments.length() > 1).then(|| crate::webidl::value_to_string(scope, arguments.get(1)));
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let found = record.pairs.iter().any(|(pair_name, pair_value)| {
        pair_name == &name && value.as_ref().is_none_or(|value| pair_value == value)
    });
    result.set(v8::Boolean::new(scope, found).into());
}

fn set(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if !require_argument_count(scope, arguments.length(), 2, "set") {
        return;
    }
    let name = crate::webidl::value_to_string(scope, arguments.get(0));
    let value = crate::webidl::value_to_string(scope, arguments.get(1));
    with_record_mut(scope, arguments.this(), |record| {
        if let Some(index) = record
            .pairs
            .iter()
            .position(|(pair_name, _)| pair_name == &name)
        {
            record.pairs[index].1 = value;
            let mut keep_first = true;
            record.pairs.retain(|(pair_name, _)| {
                if pair_name != &name {
                    true
                } else if keep_first {
                    keep_first = false;
                    true
                } else {
                    false
                }
            });
        } else {
            record.pairs.push((name, value));
        }
    });
}

fn sort(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    with_record_mut(scope, arguments.this(), |record| {
        record
            .pairs
            .sort_by(|(left, _), (right, _)| left.encode_utf16().cmp(right.encode_utf16()));
    });
}

fn to_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        let value = serialize(&record.pairs);
        if let Some(value) = v8::String::new(scope, &value) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn ensure_iterator_prototype<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(prototype) = scope
        .get_slot::<UrlSearchParamsStore>()
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
        return Err("cannot inherit URLSearchParams Iterator prototype".to_owned());
    }
    crate::trace::label_native_value_once(
        scope,
        prototype.into(),
        "URLSearchParams Iterator.prototype",
    );
    crate::webidl::define_method(scope, prototype, "next", 0, iterator_next)?;
    crate::webidl::define_to_string_tag(scope, prototype, "URLSearchParams Iterator")?;
    let stored = v8::Global::new(scope, prototype);
    scope
        .get_slot_mut::<UrlSearchParamsStore>()
        .ok_or_else(|| "URLSearchParams state was not prepared".to_owned())?
        .iterator_prototypes
        .insert(realm_id, stored);
    Ok(prototype)
}

fn create_iterator<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    params_id: i32,
    kind: ParamsIteratorKind,
) -> Result<v8::Local<'s, v8::Object>, String> {
    if !scope
        .get_slot::<UrlSearchParamsStore>()
        .is_some_and(|store| store.records.contains_key(&params_id))
    {
        return Err("Illegal invocation".to_owned());
    }
    let prototype = ensure_iterator_prototype(scope)?;
    let iterator = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, iterator, prototype.into()) != Some(true) {
        return Err("cannot create URLSearchParams Iterator".to_owned());
    }
    scope
        .get_slot_mut::<UrlSearchParamsStore>()
        .ok_or_else(|| "URLSearchParams state was not prepared".to_owned())?
        .iterators
        .insert(
            iterator.get_identity_hash().get(),
            ParamsIteratorRecord {
                params_id,
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
        .get_slot::<UrlSearchParamsStore>()
        .and_then(|store| store.iterators.get(&iterator_id))
        .cloned()
    else {
        crate::webidl::throw_type_error(
            scope,
            "Method URLSearchParams Iterator.prototype.next called on incompatible receiver",
        );
        return;
    };
    let pair = scope
        .get_slot::<UrlSearchParamsStore>()
        .and_then(|store| store.records.get(&iterator.params_id))
        .and_then(|record| record.pairs.get(iterator.index))
        .cloned();
    let Some((name, value)) = pair else {
        let undefined = v8::undefined(scope);
        result.set(iterator_result(scope, undefined.into(), true).into());
        return;
    };
    if let Some(record) = scope
        .get_slot_mut::<UrlSearchParamsStore>()
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
        ParamsIteratorKind::Entries => {
            let pair = v8::Array::new(scope, 2);
            let _ = pair.set_index(scope, 0, name.into());
            let _ = pair.set_index(scope, 1, value.into());
            pair.into()
        }
        ParamsIteratorKind::Keys => name.into(),
        ParamsIteratorKind::Values => value.into(),
    };
    result.set(iterator_result(scope, output, false).into());
}

fn return_iterator(
    scope: &mut v8::PinScope<'_, '_>,
    receiver: v8::Local<'_, v8::Object>,
    kind: ParamsIteratorKind,
    mut result: v8::ReturnValue<'_>,
) {
    let params_id = receiver.get_identity_hash().get();
    match create_iterator(scope, params_id, kind) {
        Ok(iterator) => result.set(iterator.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn entries(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    return_iterator(scope, arguments.this(), ParamsIteratorKind::Entries, result);
}

fn keys(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    return_iterator(scope, arguments.this(), ParamsIteratorKind::Keys, result);
}

fn values(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    return_iterator(scope, arguments.this(), ParamsIteratorKind::Values, result);
}

fn for_each(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    if !require_argument_count(scope, arguments.length(), 1, "forEach") {
        return;
    }
    let callback = match v8::Local::<v8::Function>::try_from(arguments.get(0)) {
        Ok(callback) => callback,
        Err(_) => {
            crate::webidl::throw_type_error(
                scope,
                "URLSearchParams.forEach callback is not callable",
            );
            return;
        }
    };
    let this_arg = arguments.get(1);
    let receiver = arguments.this();
    if record(scope, receiver).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let params_id = receiver.get_identity_hash().get();
    let mut index = 0;
    loop {
        let pair = scope
            .get_slot::<UrlSearchParamsStore>()
            .and_then(|store| store.records.get(&params_id))
            .and_then(|record| record.pairs.get(index))
            .cloned();
        let Some((name, value)) = pair else {
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
                this_arg,
                &[value.into(), name.into(), receiver.into()],
            )
            .is_none()
        {
            return;
        }
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<UrlSearchParamsStore>() {
        store.constructors.remove(&realm_id);
        store.iterator_prototypes.remove(&realm_id);
    }
}
