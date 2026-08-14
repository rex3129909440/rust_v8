use std::collections::HashMap;

#[derive(Clone)]
struct Record {
    serial: String,
    message: v8::Global<v8::Object>,
}
#[derive(Default)]
pub(crate) struct NdefReadingEventStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, Record>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(NdefReadingEventStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "NDEFReadingEvent", c.into())
}
fn ensure<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm = crate::webidl::realm_id(s);
    if let Some(v) = s
        .get_slot::<NdefReadingEventStore>()
        .and_then(|x| x.constructor.get(realm))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c = crate::webidl::create_function(
        s,
        "NDEFReadingEvent",
        2,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let parent = super::event::ensure_constructor(s)?;
    crate::webidl::inherit(s, c, parent)?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_readonly_accessor(s, p, "serialNumber", get_serial)?;
    crate::webidl::define_readonly_accessor(s, p, "message", get_message)?;
    crate::webidl::finish_constructor(s, p, c)?;
    super::android_api_support::set_tag(s, p, "NDEFReadingEvent")?;
    let stored_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<NdefReadingEventStore>()
        .unwrap()
        .constructor
        .insert(realm, stored_constructor);
    Ok(c)
}
fn construct(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !a.is_construct_call() {
        crate::webidl::throw_type_error(
            s,
            "Failed to construct 'NDEFReadingEvent': Please use the 'new' operator, this DOM object constructor cannot be called as a function.",
        );
        return;
    }
    if a.length() < 2 {
        crate::webidl::throw_type_error(
            s,
            &format!(
                "Failed to construct 'NDEFReadingEvent': 2 arguments required, but only {} present.",
                a.length()
            ),
        );
        return;
    }
    let Ok(init) = v8::Local::<v8::Object>::try_from(a.get(1)) else {
        crate::webidl::throw_type_error(s, "NDEFReadingEventInit must be an object");
        return;
    };
    let Some(message) =
        super::android_api_support::property(s, init, "message").filter(|v| !v.is_undefined())
    else {
        crate::webidl::throw_type_error(s, "NDEFReadingEventInit.message is required");
        return;
    };
    let message = if let Ok(object) = v8::Local::<v8::Object>::try_from(message) {
        match super::ndef_message::parse_init(s, object)
            .and_then(|v| super::ndef_message::create(s, v))
        {
            Ok(v) => v,
            Err(e) => {
                crate::webidl::throw_type_error(s, &e);
                return;
            }
        }
    } else {
        crate::webidl::throw_type_error(s, "NDEFReadingEventInit.message must be an object");
        return;
    };
    let serial = super::android_api_support::property(s, init, "serialNumber")
        .filter(|v| !v.is_null() && !v.is_undefined())
        .map(|v| crate::webidl::value_to_string(s, v))
        .unwrap_or_default();
    let event_type = crate::webidl::value_to_string(s, a.get(0));
    super::event::attach(s, a.this(), event_type, false, false, false);
    let stored_message = v8::Global::new(s, message);
    s.get_slot_mut::<NdefReadingEventStore>()
        .unwrap()
        .records
        .insert(
            a.this().get_identity_hash().get(),
            Record {
                serial,
                message: stored_message,
            },
        );
    r.set(a.this().into());
}
fn record(s: &mut v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<Record> {
    let v = s
        .get_slot::<NdefReadingEventStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned();
    if v.is_none() {
        crate::webidl::throw_type_error(s, "Illegal invocation");
    }
    v
}
fn get_serial(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(x) = record(s, a.this()) {
        if let Some(v) = v8::String::new(s, &x.serial) {
            r.set(v.into());
        }
    }
}
fn get_message(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(x) = record(s, a.this()) {
        r.set(v8::Local::new(s, &x.message).into());
    }
}
