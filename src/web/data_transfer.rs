use std::collections::HashMap;

#[derive(Clone)]
struct DataTransferRecord {
    drop_effect: String,
    effect_allowed: String,
    items: v8::Global<v8::Object>,
    drag_image: Option<(v8::Global<v8::Object>, i32, i32)>,
    access_mode: AccessMode,
    target_view: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum AccessMode {
    ReadWrite,
    ReadOnly,
    Protected,
}

#[derive(Default)]
pub(crate) struct DataTransferStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, DataTransferRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(DataTransferStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "DataTransfer", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<DataTransferStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "DataTransfer",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "dropEffect",
        get_drop_effect,
        set_drop_effect,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "effectAllowed",
        get_effect_allowed,
        set_effect_allowed,
    )?;
    crate::webidl::define_readonly_accessor(scope, prototype, "items", get_items)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "types", get_types)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "files", get_files)?;
    crate::webidl::define_method(scope, prototype, "clearData", 0, clear_data)?;
    crate::webidl::define_method(scope, prototype, "getData", 1, get_data)?;
    crate::webidl::define_method(scope, prototype, "setData", 2, set_data)?;
    crate::webidl::define_method(scope, prototype, "setDragImage", 3, set_drag_image)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<DataTransferStore>()
        .ok_or_else(|| "DataTransfer state was not prepared".to_owned())?
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
        crate::webidl::throw_type_error(scope, "DataTransfer must be constructed with new");
        return;
    }
    let items = match super::data_transfer_item_list::create(scope) {
        Ok(items) => items,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    let items = v8::Global::new(scope, items);
    scope
        .get_slot_mut::<DataTransferStore>()
        .expect("DataTransfer state")
        .records
        .insert(
            arguments.this().get_identity_hash().get(),
            DataTransferRecord {
                drop_effect: "none".to_owned(),
                effect_allowed: "none".to_owned(),
                items,
                drag_image: None,
                access_mode: AccessMode::ReadWrite,
                target_view: false,
            },
        );
    result.set(arguments.this().into());
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<DataTransferRecord> {
    scope
        .get_slot::<DataTransferStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn is_instance(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> bool {
    scope.get_slot::<DataTransferStore>().is_some_and(|store| {
        store
            .records
            .contains_key(&object.get_identity_hash().get())
    })
}

pub(crate) fn create_for_drag<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let transfer = constructor
        .new_instance(scope, &[])
        .ok_or_else(|| "cannot create drag DataTransfer".to_owned())?;
    update(scope, transfer, |record| {
        record.drop_effect = "none".to_owned();
        record.effect_allowed = "uninitialized".to_owned();
        record.access_mode = AccessMode::Protected;
        record.target_view = false;
    });
    Ok(transfer)
}

pub(crate) fn set_target_view(
    scope: &mut v8::PinScope<'_, '_>,
    transfer: v8::Local<'_, v8::Object>,
    target_view: bool,
) {
    update(scope, transfer, |record| record.target_view = target_view);
}

pub(crate) fn set_access_mode(
    scope: &mut v8::PinScope<'_, '_>,
    transfer: v8::Local<'_, v8::Object>,
    access_mode: AccessMode,
) {
    update(scope, transfer, |record| record.access_mode = access_mode);
}

pub(crate) fn item_list_access_mode(
    scope: &v8::PinScope<'_, '_>,
    list: v8::Local<'_, v8::Object>,
) -> Option<AccessMode> {
    let list_id = list.get_identity_hash();
    scope
        .get_slot::<DataTransferStore>()?
        .records
        .values()
        .find(|record| v8::Local::new(scope, &record.items).get_identity_hash() == list_id)
        .map(|record| record.access_mode)
}

pub(crate) fn set_drag_effects(
    scope: &mut v8::PinScope<'_, '_>,
    transfer: v8::Local<'_, v8::Object>,
    drop_effect: Option<&str>,
    effect_allowed: Option<&str>,
) {
    update(scope, transfer, |record| {
        if let Some(value) = drop_effect {
            record.drop_effect = value.to_owned();
        }
        if let Some(value) = effect_allowed {
            record.effect_allowed = value.to_owned();
        }
    });
}

fn get_drop_effect(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    return_string(scope, arguments, result, |record| record.drop_effect)
}

fn get_effect_allowed(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    return_string(scope, arguments, result, |record| record.effect_allowed)
}

fn return_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(DataTransferRecord) -> String,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(value) = v8::String::new(scope, &select(record)) {
        result.set(value.into());
    }
}

fn set_drop_effect(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    if !matches!(value.as_str(), "none" | "copy" | "link" | "move") {
        return;
    }
    update(scope, arguments.this(), |record| record.drop_effect = value);
}

fn set_effect_allowed(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    if !matches!(
        value.as_str(),
        "none"
            | "copy"
            | "copyLink"
            | "copyMove"
            | "link"
            | "linkMove"
            | "move"
            | "all"
            | "uninitialized"
    ) {
        return;
    }
    update(scope, arguments.this(), |record| {
        record.effect_allowed = value
    });
}

fn update(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut DataTransferRecord),
) {
    if let Some(record) = scope
        .get_slot_mut::<DataTransferStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        change(record);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_items(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.items).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn item_records<'s>(
    scope: &v8::PinScope<'s, '_>,
    record: &DataTransferRecord,
) -> Vec<(
    v8::Local<'s, v8::Object>,
    super::data_transfer_item::ItemRecord,
)> {
    let list = v8::Local::new(scope, &record.items);
    super::data_transfer_item_list::items(scope, list)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|item| {
            let record = super::data_transfer_item::record(scope, item)?;
            Some((item, record))
        })
        .collect()
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
    let mut string_types = Vec::new();
    let mut has_files = false;
    for (_, item) in item_records(scope, &record) {
        match item.payload {
            super::data_transfer_item::ItemPayload::String(_) => {
                if !string_types
                    .iter()
                    .any(|(media_type, _)| media_type == &item.media_type)
                {
                    string_types.push((item.media_type, item.created_via_set_data));
                }
            }
            super::data_transfer_item::ItemPayload::File(_) => has_files = true,
        }
    }
    if record.target_view {
        string_types.sort_by_key(|(media_type, created_via_set_data)| {
            if media_type == "text/plain" {
                0
            } else if !created_via_set_data {
                1
            } else {
                2
            }
        });
    }
    let mut types: Vec<String> = string_types
        .into_iter()
        .map(|(media_type, _)| media_type)
        .collect();
    if has_files {
        types.push("Files".to_owned());
    }
    let array = v8::Array::new(scope, types.len() as i32);
    for (index, value) in types.iter().enumerate() {
        if let Some(value) = v8::String::new(scope, value) {
            let _ = array.set_index(scope, index as u32, value.into());
        }
    }
    let _ = array.set_integrity_level(scope, v8::IntegrityLevel::Frozen);
    result.set(array.into());
}

fn get_files(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let files = item_records(scope, &record)
        .into_iter()
        .filter_map(|(_, item)| match item.payload {
            super::data_transfer_item::ItemPayload::File(file) => {
                Some(v8::Local::new(scope, &file))
            }
            _ => None,
        })
        .collect();
    match super::file_list::create(scope, files) {
        Ok(files) => result.set(files.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn normalize_type(value: String) -> String {
    match value.to_ascii_lowercase().as_str() {
        "text" => "text/plain".to_owned(),
        "url" => "text/uri-list".to_owned(),
        value => value.to_owned(),
    }
}

fn clear_data(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if record.access_mode != AccessMode::ReadWrite {
        return;
    }
    let list = v8::Local::new(scope, &record.items);
    if arguments.length() == 0 {
        super::data_transfer_item_list::clear_strings(scope, list);
    } else {
        let media_type = normalize_type(crate::webidl::value_to_string(scope, arguments.get(0)));
        super::data_transfer_item_list::clear_type(scope, list, &media_type);
    }
}

fn get_data(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if record.access_mode == AccessMode::Protected {
        if let Some(value) = v8::String::new(scope, "") {
            result.set(value.into());
        }
        return;
    }
    let media_type = normalize_type(crate::webidl::value_to_string(scope, arguments.get(0)));
    let value = item_records(scope, &record)
        .into_iter()
        .find_map(|(_, item)| {
            if item.media_type == media_type
                && let super::data_transfer_item::ItemPayload::String(value) = item.payload
            {
                return Some(value);
            }
            None
        })
        .unwrap_or_default();
    if let Some(value) = v8::String::new(scope, &value) {
        result.set(value.into());
    }
}

fn set_data(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if record.access_mode != AccessMode::ReadWrite {
        return;
    }
    let media_type = normalize_type(crate::webidl::value_to_string(scope, arguments.get(0)));
    let value = crate::webidl::value_to_string(scope, arguments.get(1));
    let list = v8::Local::new(scope, &record.items);
    if let Err(message) = super::data_transfer_item_list::set_string(scope, list, media_type, value)
    {
        crate::webidl::throw_type_error(scope, &message);
    }
}

fn set_drag_image(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(transfer) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let Ok(image) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "The drag image must be an Element");
        return;
    };
    if transfer.access_mode != AccessMode::ReadWrite {
        return;
    }
    if super::node::record(scope, image).is_none_or(|record| record.node_type != 1) {
        crate::webidl::throw_type_error(scope, "The drag image must be an Element");
        return;
    }
    let x = arguments.get(1).int32_value(scope).unwrap_or(0);
    let y = arguments.get(2).int32_value(scope).unwrap_or(0);
    let image = v8::Global::new(scope, image);
    update(scope, arguments.this(), |record| {
        record.drag_image = Some((image, x, y))
    });
}
