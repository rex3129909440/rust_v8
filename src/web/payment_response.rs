use std::collections::HashMap;
#[derive(Clone)]
struct ResponseRecord {
    request_id: String,
    method: String,
    details: v8::Global<v8::Object>,
    handler: Option<v8::Global<v8::Value>>,
    completed: bool,
}
#[derive(Default)]
pub(crate) struct PaymentResponseStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, ResponseRecord>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(PaymentResponseStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "PaymentResponse", c.into())
}
fn ensure<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(v) = s
        .get_slot::<PaymentResponseStore>()
        .and_then(|x| x.constructor.get(crate::webidl::realm_id(s)))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c = crate::webidl::create_function(
        s,
        "PaymentResponse",
        0,
        v8::ConstructorBehavior::Allow,
        illegal,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_readonly_accessor(s, p, "requestId", request_id)?;
    crate::webidl::define_readonly_accessor(s, p, "methodName", method_name)?;
    crate::webidl::define_readonly_accessor(s, p, "details", details)?;
    crate::webidl::define_readonly_accessor(s, p, "shippingAddress", nullable)?;
    crate::webidl::define_readonly_accessor(s, p, "shippingOption", nullable)?;
    crate::webidl::define_readonly_accessor(s, p, "payerName", nullable)?;
    crate::webidl::define_readonly_accessor(s, p, "payerEmail", nullable)?;
    crate::webidl::define_readonly_accessor(s, p, "payerPhone", nullable)?;
    crate::webidl::define_accessor(s, p, "onpayerdetailchange", get_handler, set_handler)?;
    crate::webidl::define_method(s, p, "complete", 0, complete)?;
    crate::webidl::define_method(s, p, "retry", 0, retry)?;
    crate::webidl::define_method(s, p, "toJSON", 0, to_json)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let parent = super::event_target::ensure_constructor(s)?;
    crate::webidl::inherit(s, c, parent)?;
    let realm_id = crate::webidl::realm_id(s);
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<PaymentResponseStore>()
        .unwrap()
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
fn illegal(
    s: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(s, "Illegal constructor")
}
pub(crate) fn create<'s>(
    s: &mut v8::PinScope<'s, '_>,
    request_id: String,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure(s)?;
    let p = crate::webidl::prototype(s, c)?;
    let o = v8::Object::new(s);
    if crate::webidl::set_platform_prototype(s, o, p.into()) != Some(true) {
        return Err("cannot create PaymentResponse".to_owned());
    }
    super::event_target::attach(s, o);
    let details = v8::Global::new(s, v8::Object::new(s));
    s.get_slot_mut::<PaymentResponseStore>()
        .unwrap()
        .records
        .insert(
            o.get_identity_hash().get(),
            ResponseRecord {
                request_id,
                method: "basic-card".to_owned(),
                details,
                handler: None,
                completed: false,
            },
        );
    Ok(o)
}
fn record(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<ResponseRecord> {
    s.get_slot::<PaymentResponseStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
fn text(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
    f: impl FnOnce(ResponseRecord) -> String,
) {
    if let Some(v) = record(s, a.this())
        && let Some(x) = v8::String::new(s, &f(v))
    {
        r.set(x.into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn request_id(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    text(s, a, r, |v| v.request_id)
}
fn method_name(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    text(s, a, r, |v| v.method)
}
fn details(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Local::new(s, &v.details).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn nullable(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_some() {
        r.set(v8::null(s).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_handler(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let Some(record) = record(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    super::window_event_handler_support::return_handler(s, record.handler, r)
}
fn set_handler(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let h = super::window_event_handler_support::handler_value(s, a.get(0));
    if let Some(v) = s
        .get_slot_mut::<PaymentResponseStore>()
        .and_then(|x| x.records.get_mut(&a.this().get_identity_hash().get()))
    {
        v.handler = h
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn void(s: &mut v8::PinScope<'_, '_>, mut r: v8::ReturnValue<'_>) {
    let x = v8::undefined(s);
    if let Ok(p) = super::writable_stream::resolved_promise(s, x.into()) {
        r.set(p.into())
    }
}
fn complete(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = s
        .get_slot_mut::<PaymentResponseStore>()
        .and_then(|x| x.records.get_mut(&a.this().get_identity_hash().get()))
    {
        v.completed = true;
        void(s, r)
    } else {
        crate::webidl::reject_illegal_invocation_promise(s, "PaymentResponse", "complete", r)
    }
}
fn retry(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_some() {
        void(s, r)
    } else {
        crate::webidl::reject_illegal_invocation_promise(s, "PaymentResponse", "retry", r)
    }
}
fn to_json(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(v) = record(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    let o = v8::Object::new(s);
    if let (Some(k), Some(x)) = (
        v8::String::new(s, "requestId"),
        v8::String::new(s, &v.request_id),
    ) {
        let _ = o.set(s, k.into(), x.into());
    }
    r.set(o.into())
}
