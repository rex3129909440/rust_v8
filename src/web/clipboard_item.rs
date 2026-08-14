use std::collections::HashMap;

#[derive(Clone)]
struct ClipboardItemRecord {
    values: HashMap<String, v8::Global<v8::Value>>,
}

#[derive(Default)]
pub(crate) struct ClipboardItemStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, ClipboardItemRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(ClipboardItemStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "ClipboardItem", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<ClipboardItemStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "ClipboardItem",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "types", get_types)?;
    crate::webidl::define_method(scope, prototype, "getType", 1, get_type)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::define_method(scope, constructor.into(), "supports", 1, supports)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<ClipboardItemStore>()
        .ok_or_else(|| "ClipboardItem state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(scope, "ClipboardItem must be constructed");
        return;
    }
    let Ok(source) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'ClipboardItem': Only objects can be converted to record<K,V> types",
        );
        return;
    };
    let mut values = HashMap::new();
    if let Some(names) = source.get_own_property_names(
        scope,
        v8::GetPropertyNamesArgs {
            mode: v8::KeyCollectionMode::OwnOnly,
            property_filter: v8::PropertyFilter::ONLY_ENUMERABLE,
            index_filter: v8::IndexFilter::IncludeIndices,
            key_conversion: v8::KeyConversionMode::ConvertToString,
        },
    ) {
        for index in 0..names.length() {
            let Some(key) = names.get_index(scope, index) else {
                continue;
            };
            let Some(value) = source.get(scope, key) else {
                continue;
            };
            values.insert(
                crate::webidl::value_to_string(scope, key),
                v8::Global::new(scope, value),
            );
        }
    }
    if values.is_empty() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'ClipboardItem': Empty dictionary argument",
        );
        return;
    }
    scope
        .get_slot_mut::<ClipboardItemStore>()
        .expect("ClipboardItem state")
        .records
        .insert(
            arguments.this().get_identity_hash().get(),
            ClipboardItemRecord { values },
        );
    result.set(arguments.this().into());
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<ClipboardItemRecord> {
    scope
        .get_slot::<ClipboardItemStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_types(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let names = record.values.keys().cloned().collect::<Vec<_>>();
    let array = v8::Array::new(scope, names.len() as i32);
    for (index, name) in names.iter().enumerate() {
        if let Some(value) = v8::String::new(scope, name) {
            let _ = array.set_index(scope, index as u32, value.into());
        }
    }
    result.set(array.into());
}

fn get_type(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::reject_illegal_invocation_promise(scope, "ClipboardItem", "getType", result);
        return;
    };
    let media_type = crate::webidl::value_to_string(scope, arguments.get(0));
    let Some(value) = record.values.get(&media_type) else {
        if let Ok(error) = super::dom_exception::create(
            scope,
            "The requested clipboard type is unavailable.".to_owned(),
            "NotFoundError".to_owned(),
        ) && let Ok(promise) = super::writable_stream::rejected_promise(scope, error.into())
        {
            result.set(promise.into());
        }
        return;
    };
    let value = v8::Local::new(scope, value);
    let blob = if value.is_object() {
        v8::Local::<v8::Object>::try_from(value).ok()
    } else {
        super::blob::create(
            scope,
            crate::webidl::value_to_string(scope, value).into_bytes(),
            &media_type,
        )
        .ok()
    };
    if let Some(blob) = blob
        && let Ok(promise) = super::writable_stream::resolved_promise(scope, blob.into())
    {
        result.set(promise.into());
    }
}

fn supports(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let media_type = crate::webidl::value_to_string(scope, arguments.get(0));
    let supported = media_type == "text/plain"
        || media_type == "text/html"
        || media_type == "image/png"
        || media_type == "image/svg+xml";
    result.set(v8::Boolean::new(scope, supported).into());
}
