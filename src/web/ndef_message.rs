use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct NdefMessageStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, Vec<super::ndef_record::NdefRecordData>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(NdefMessageStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(scope)?;
    crate::webidl::define_global(scope, "NDEFMessage", c.into())
}
fn ensure<'s>(scope: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm = crate::webidl::realm_id(scope);
    if let Some(v) = scope
        .get_slot::<NdefMessageStore>()
        .and_then(|s| s.constructor.get(realm))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &v));
    }
    let c = crate::webidl::create_function(
        scope,
        "NDEFMessage",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let p = crate::webidl::prototype(scope, c)?;
    crate::webidl::reset_constructor_order(scope, p)?;
    crate::webidl::define_readonly_accessor(scope, p, "records", get_records)?;
    crate::webidl::finish_constructor(scope, p, c)?;
    super::android_api_support::set_tag(scope, p, "NDEFMessage")?;
    let stored_constructor = v8::Global::new(scope, c);
    scope
        .get_slot_mut::<NdefMessageStore>()
        .unwrap()
        .constructor
        .insert(realm, stored_constructor);
    Ok(c)
}
fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !a.is_construct_call() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'NDEFMessage': Please use the 'new' operator, this DOM object constructor cannot be called as a function.",
        );
        return;
    }
    if a.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'NDEFMessage': 1 argument required, but only 0 present.",
        );
        return;
    }
    let Ok(init) = v8::Local::<v8::Object>::try_from(a.get(0)) else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'NDEFMessage': parameter 1 is not of type 'NDEFMessageInit'.",
        );
        return;
    };
    let values = match parse_init(scope, init) {
        Ok(v) => v,
        Err(e) => {
            crate::webidl::throw_type_error(scope, &e);
            return;
        }
    };
    scope
        .get_slot_mut::<NdefMessageStore>()
        .unwrap()
        .records
        .insert(a.this().get_identity_hash().get(), values);
    r.set(a.this().into());
}
pub(crate) fn parse_init(
    scope: &mut v8::PinScope<'_, '_>,
    init: v8::Local<'_, v8::Object>,
) -> Result<Vec<super::ndef_record::NdefRecordData>, String> {
    let value=super::android_api_support::property(scope,init,"records").filter(|v|!v.is_undefined()).ok_or_else(||"Failed to construct 'NDEFMessage': Failed to read the 'records' property from 'NDEFMessageInit': Required member is undefined.".to_owned())?;
    let array = v8::Local::<v8::Array>::try_from(value)
        .map_err(|_| "NDEFMessageInit.records must be a sequence".to_owned())?;
    let mut out = Vec::new();
    for i in 0..array.length() {
        let value = array
            .get_index(scope, i)
            .ok_or_else(|| "invalid NDEF record".to_owned())?;
        let object = v8::Local::<v8::Object>::try_from(value)
            .map_err(|_| "NDEF record must be an object".to_owned())?;
        let c = ensure_record_from_init(scope, object)?;
        out.push(c);
    }
    Ok(out)
}
fn ensure_record_from_init(
    scope: &mut v8::PinScope<'_, '_>,
    init: v8::Local<'_, v8::Object>,
) -> Result<super::ndef_record::NdefRecordData, String> {
    if let Some(record) = super::ndef_record::snapshot(scope, init) {
        return Ok(record);
    }
    let rt = super::android_api_support::property(scope, init, "recordType")
        .filter(|v| !v.is_undefined())
        .ok_or_else(|| "NDEFRecordInit.recordType is required".to_owned())?;
    let opt = |name| {
        super::android_api_support::property(scope, init, name)
            .filter(|v| !v.is_undefined())
            .map(|v| crate::webidl::value_to_string(scope, v))
    };
    let rt = crate::webidl::value_to_string(scope, rt);
    let bytes = super::android_api_support::property(scope, init, "data")
        .filter(|v| !v.is_undefined())
        .map(|v| crate::webidl::value_to_string(scope, v).into_bytes());
    Ok(super::ndef_record::NdefRecordData {
        record_type: rt,
        media_type: opt("mediaType"),
        id: opt("id"),
        encoding: opt("encoding").or_else(|| Some("utf-8".to_owned())),
        lang: opt("lang"),
        bytes,
        nested: None,
    })
}
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    records: Vec<super::ndef_record::NdefRecordData>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure(scope)?;
    let p = crate::webidl::prototype(scope, c)?;
    let o = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, o, p.into()) != Some(true) {
        return Err("cannot create NDEFMessage".to_owned());
    }
    scope
        .get_slot_mut::<NdefMessageStore>()
        .unwrap()
        .records
        .insert(o.get_identity_hash().get(), records);
    Ok(o)
}
fn get_records(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(records) = scope
        .get_slot::<NdefMessageStore>()
        .and_then(|s| s.records.get(&a.this().get_identity_hash().get()))
        .cloned()
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let values = v8::Array::new(scope, records.len() as i32);
    for (i, record) in records.into_iter().enumerate() {
        if let Ok(record) = super::ndef_record::create(scope, record) {
            let _ = values.set_index(scope, i as u32, record.into());
        }
    }
    let _ = values.set_integrity_level(scope, v8::IntegrityLevel::Frozen);
    r.set(values.into());
}
