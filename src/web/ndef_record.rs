use std::collections::HashMap;

#[derive(Clone)]
pub(crate) struct NdefRecordData {
    pub(crate) record_type: String,
    pub(crate) media_type: Option<String>,
    pub(crate) id: Option<String>,
    pub(crate) encoding: Option<String>,
    pub(crate) lang: Option<String>,
    pub(crate) bytes: Option<Vec<u8>>,
    pub(crate) nested: Option<Vec<NdefRecordData>>,
}

#[derive(Default)]
pub(crate) struct NdefRecordStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, NdefRecordData>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(NdefRecordStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure(scope)?;
    crate::webidl::define_global(scope, "NDEFRecord", constructor.into())
}

fn ensure<'s>(scope: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm = crate::webidl::realm_id(scope);
    if let Some(value) = scope
        .get_slot::<NdefRecordStore>()
        .and_then(|s| s.constructor.get(realm))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &value));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "NDEFRecord",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "recordType", get_record_type)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "mediaType", get_media_type)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "id", get_id)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "encoding", get_encoding)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "lang", get_lang)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "data", get_data)?;
    crate::webidl::define_method(scope, prototype, "toRecords", 0, to_records)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    super::android_api_support::set_tag(scope, prototype, "NDEFRecord")?;
    let stored_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<NdefRecordStore>()
        .unwrap()
        .constructor
        .insert(realm, stored_constructor);
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
            "Failed to construct 'NDEFRecord': Please use the 'new' operator, this DOM object constructor cannot be called as a function.",
        );
        return;
    }
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'NDEFRecord': 1 argument required, but only 0 present.",
        );
        return;
    }
    let Ok(init) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'NDEFRecord': parameter 1 is not of type 'NDEFRecordInit'.",
        );
        return;
    };
    let Some(record_type_value) = super::android_api_support::property(scope, init, "recordType")
        .filter(|v| !v.is_undefined())
    else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'NDEFRecord': Failed to read the 'recordType' property from 'NDEFRecordInit': Required member is undefined.",
        );
        return;
    };
    let record_type = crate::webidl::value_to_string(scope, record_type_value);
    let media_type = super::android_api_support::property(scope, init, "mediaType")
        .filter(|v| !v.is_undefined())
        .map(|v| crate::webidl::value_to_string(scope, v));
    let id = super::android_api_support::property(scope, init, "id")
        .filter(|v| !v.is_undefined())
        .map(|v| crate::webidl::value_to_string(scope, v));
    let encoding = super::android_api_support::property(scope, init, "encoding")
        .filter(|v| !v.is_undefined())
        .map(|v| crate::webidl::value_to_string(scope, v));
    let lang = super::android_api_support::property(scope, init, "lang")
        .filter(|v| !v.is_undefined())
        .map(|v| crate::webidl::value_to_string(scope, v));
    let mut bytes = None;
    let mut nested = None;
    if let Some(data) =
        super::android_api_support::property(scope, init, "data").filter(|v| !v.is_undefined())
    {
        if let Ok(view) = v8::Local::<v8::ArrayBufferView>::try_from(data) {
            let mut value = vec![0; view.byte_length()];
            let length = view.copy_contents(&mut value);
            value.truncate(length);
            bytes = Some(value);
        } else if let Ok(buffer) = v8::Local::<v8::ArrayBuffer>::try_from(data) {
            let backing = buffer.get_backing_store();
            let value = backing
                .data()
                .map(|data| {
                    unsafe {
                        std::slice::from_raw_parts(
                            data.as_ptr().cast::<u8>(),
                            backing.byte_length(),
                        )
                    }
                    .to_vec()
                })
                .unwrap_or_default();
            bytes = Some(value);
        } else if record_type == "smart-poster" {
            if let Ok(object) = v8::Local::<v8::Object>::try_from(data) {
                nested = super::ndef_message::parse_init(scope, object).ok();
            }
        } else {
            bytes = Some(crate::webidl::value_to_string(scope, data).into_bytes());
        }
    }
    let data = NdefRecordData {
        record_type,
        media_type,
        id,
        encoding: encoding.or_else(|| Some("utf-8".to_owned())),
        lang,
        bytes,
        nested,
    };
    scope
        .get_slot_mut::<NdefRecordStore>()
        .unwrap()
        .records
        .insert(arguments.this().get_identity_hash().get(), data);
    result.set(arguments.this().into());
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    data: NdefRecordData,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create NDEFRecord".to_owned());
    }
    scope
        .get_slot_mut::<NdefRecordStore>()
        .unwrap()
        .records
        .insert(object.get_identity_hash().get(), data);
    Ok(object)
}

pub(crate) fn snapshot(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<NdefRecordData> {
    scope
        .get_slot::<NdefRecordStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn record(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<NdefRecordData> {
    let value = snapshot(scope, object);
    if value.is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
    value
}

fn return_optional(
    scope: &mut v8::PinScope<'_, '_>,
    value: Option<&str>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(value) = value {
        if let Some(value) = v8::String::new(scope, value) {
            result.set(value.into());
        }
    } else {
        result.set(v8::null(scope).into());
    }
}
fn get_record_type(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let value = record(s, a.this());
    return_optional(s, value.as_ref().map(|x| x.record_type.as_str()), r);
}
fn get_media_type(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let value = record(s, a.this());
    return_optional(s, value.as_ref().and_then(|x| x.media_type.as_deref()), r);
}
fn get_id(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let value = record(s, a.this());
    return_optional(s, value.as_ref().and_then(|x| x.id.as_deref()), r);
}
fn get_encoding(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let value = record(s, a.this());
    return_optional(s, value.as_ref().and_then(|x| x.encoding.as_deref()), r);
}
fn get_lang(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let value = record(s, a.this());
    return_optional(s, value.as_ref().and_then(|x| x.lang.as_deref()), r);
}
fn get_data(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(value) = record(scope, arguments.this()) else {
        return;
    };
    let Some(bytes) = value.bytes else {
        result.set(v8::null(scope).into());
        return;
    };
    let length = bytes.len();
    let backing = v8::ArrayBuffer::new_backing_store_from_vec(bytes).make_shared();
    let buffer = v8::ArrayBuffer::with_backing_store(scope, &backing);
    result.set(v8::DataView::new(scope, buffer, 0, length).into());
}
fn to_records(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(value) = record(scope, arguments.this()) else {
        return;
    };
    let Some(records) = value.nested else {
        result.set(v8::null(scope).into());
        return;
    };
    let array = v8::Array::new(scope, records.len() as i32);
    for (index, record) in records.into_iter().enumerate() {
        if let Ok(record) = create(scope, record) {
            let _ = array.set_index(scope, index as u32, record.into());
        }
    }
    result.set(array.into());
}
