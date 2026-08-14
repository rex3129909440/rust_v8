use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct DataTransferItemListStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, Vec<v8::Global<v8::Object>>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(DataTransferItemListStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "DataTransferItemList", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<DataTransferItemListStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "DataTransferItemList",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "length", get_length)?;
    crate::webidl::define_method(scope, prototype, "add", 1, add)?;
    crate::webidl::define_method(scope, prototype, "clear", 0, clear)?;
    crate::webidl::define_method(scope, prototype, "remove", 1, remove)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::define_indexed_iterator(scope, prototype)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<DataTransferItemListStore>()
        .ok_or_else(|| "DataTransferItemList state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let list = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, list, prototype.into()) != Some(true) {
        return Err("cannot create DataTransferItemList".to_owned());
    }
    scope
        .get_slot_mut::<DataTransferItemListStore>()
        .ok_or_else(|| "DataTransferItemList state was not prepared".to_owned())?
        .records
        .insert(list.get_identity_hash().get(), Vec::new());
    Ok(list)
}

fn records(
    scope: &v8::PinScope<'_, '_>,
    list: v8::Local<'_, v8::Object>,
) -> Option<Vec<v8::Global<v8::Object>>> {
    scope
        .get_slot::<DataTransferItemListStore>()?
        .records
        .get(&list.get_identity_hash().get())
        .cloned()
}

pub(crate) fn items<'s>(
    scope: &v8::PinScope<'s, '_>,
    list: v8::Local<'_, v8::Object>,
) -> Option<Vec<v8::Local<'s, v8::Object>>> {
    Some(
        records(scope, list)?
            .iter()
            .map(|item| v8::Local::new(scope, item))
            .collect(),
    )
}

fn replace_records(
    scope: &mut v8::PinScope<'_, '_>,
    list: v8::Local<'_, v8::Object>,
    values: Vec<v8::Global<v8::Object>>,
) -> bool {
    let old_length = records(scope, list).map_or(0, |values| values.len());
    let Some(store) = scope.get_slot_mut::<DataTransferItemListStore>() else {
        return false;
    };
    if !store.records.contains_key(&list.get_identity_hash().get()) {
        return false;
    }
    store
        .records
        .insert(list.get_identity_hash().get(), values.clone());
    for index in 0..old_length {
        let _ = list.delete_index(scope, index as u32);
    }
    for (index, item) in values.iter().enumerate() {
        let item = v8::Local::new(scope, item);
        let Some(key) = v8::String::new(scope, &index.to_string()) else {
            continue;
        };
        let _ = list.define_own_property(
            scope,
            key.into(),
            item.into(),
            v8::PropertyAttribute::READ_ONLY | v8::PropertyAttribute::DONT_DELETE,
        );
    }
    true
}

pub(crate) fn clear_all(scope: &mut v8::PinScope<'_, '_>, list: v8::Local<'_, v8::Object>) -> bool {
    replace_records(scope, list, Vec::new())
}

pub(crate) fn clear_strings(
    scope: &mut v8::PinScope<'_, '_>,
    list: v8::Local<'_, v8::Object>,
) -> bool {
    let Some(values) = records(scope, list) else {
        return false;
    };
    let retained = values
        .into_iter()
        .filter(|item| {
            let item = v8::Local::new(scope, item);
            super::data_transfer_item::record(scope, item).is_some_and(|record| {
                matches!(
                    record.payload,
                    super::data_transfer_item::ItemPayload::File(_)
                )
            })
        })
        .collect();
    replace_records(scope, list, retained)
}

pub(crate) fn clear_type(
    scope: &mut v8::PinScope<'_, '_>,
    list: v8::Local<'_, v8::Object>,
    media_type: &str,
) -> bool {
    let Some(values) = records(scope, list) else {
        return false;
    };
    let retained = values
        .into_iter()
        .filter(|item| {
            let item = v8::Local::new(scope, item);
            super::data_transfer_item::record(scope, item)
                .is_none_or(|record| !record.media_type.eq_ignore_ascii_case(media_type))
        })
        .collect();
    replace_records(scope, list, retained)
}

pub(crate) fn set_string<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    list: v8::Local<'_, v8::Object>,
    media_type: String,
    value: String,
) -> Result<v8::Local<'s, v8::Object>, String> {
    clear_type(scope, list, &media_type);
    let item = super::data_transfer_item::create_string_from_set_data(scope, value, media_type)?;
    let mut values =
        records(scope, list).ok_or_else(|| "Invalid DataTransferItemList".to_owned())?;
    values.push(v8::Global::new(scope, item));
    replace_records(scope, list, values);
    Ok(item)
}

fn file_media_type(
    scope: &v8::PinScope<'_, '_>,
    file: v8::Local<'_, v8::Object>,
) -> Option<String> {
    super::blob::byte_snapshot(scope, file)?;
    let name_key = v8::String::new(scope, "name")?;
    let name = file.get(scope, name_key.into())?;
    if name.is_undefined() {
        return None;
    }
    let type_key = v8::String::new(scope, "type")?;
    let media_type = file.get(scope, type_key.into())?;
    Some(crate::webidl::value_to_string(scope, media_type))
}

fn add(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(existing) = records(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if super::data_transfer::item_list_access_mode(scope, arguments.this())
        != Some(super::data_transfer::AccessMode::ReadWrite)
    {
        result.set(v8::null(scope).into());
        return;
    }
    let created = if let Ok(file) = v8::Local::<v8::Object>::try_from(arguments.get(0))
        && let Some(media_type) = file_media_type(scope, file)
    {
        super::data_transfer_item::create_file(scope, file, media_type)
    } else {
        if arguments.length() < 2 {
            crate::webidl::throw_type_error(scope, "A string item requires a media type");
            return;
        }
        let value = crate::webidl::value_to_string(scope, arguments.get(0));
        let media_type =
            crate::webidl::value_to_string(scope, arguments.get(1)).to_ascii_lowercase();
        let duplicate = existing.iter().any(|item| {
            let item = v8::Local::new(scope, item);
            super::data_transfer_item::record(scope, item).is_some_and(|record| {
                matches!(
                    record.payload,
                    super::data_transfer_item::ItemPayload::String(_)
                ) && record.media_type == media_type
            })
        });
        if duplicate {
            crate::webidl::throw_type_error(scope, "An item of this type already exists");
            return;
        }
        super::data_transfer_item::create_string(scope, value, media_type)
    };
    match created {
        Ok(item) => {
            let mut updated = existing;
            updated.push(v8::Global::new(scope, item));
            replace_records(scope, arguments.this(), updated);
            result.set(item.into());
        }
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn clear(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if records(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    if super::data_transfer::item_list_access_mode(scope, arguments.this())
        != Some(super::data_transfer::AccessMode::ReadWrite)
    {
        return;
    }
    if !clear_all(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn remove(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(mut values) = records(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if super::data_transfer::item_list_access_mode(scope, arguments.this())
        != Some(super::data_transfer::AccessMode::ReadWrite)
    {
        throw_invalid_state(scope);
        return;
    }
    let index = arguments.get(0).uint32_value(scope).unwrap_or(u32::MAX) as usize;
    if index < values.len() {
        values.remove(index);
        replace_records(scope, arguments.this(), values);
    }
}

fn throw_invalid_state(scope: &mut v8::PinScope<'_, '_>) {
    let exception = v8::Exception::error(
        scope,
        v8::String::new(scope, "The list is not writable.").expect("static string"),
    );
    if let Ok(object) = v8::Local::<v8::Object>::try_from(exception) {
        let _ = object.set(
            scope,
            v8::String::new(scope, "name")
                .expect("static string")
                .into(),
            v8::String::new(scope, "InvalidStateError")
                .expect("static string")
                .into(),
        );
    }
    scope.throw_exception(exception);
}

fn get_length(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(values) = records(scope, arguments.this()) {
        result.set(v8::Integer::new_from_unsigned(scope, values.len() as u32).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor");
}
