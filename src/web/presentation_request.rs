use std::collections::HashMap;
#[derive(Clone)]
struct RequestRecord {
    url: String,
    handler: Option<v8::Global<v8::Value>>,
}
#[derive(Default)]
pub(crate) struct PresentationRequestStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, RequestRecord>,
    next: u64,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(PresentationRequestStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "PresentationRequest", c.into())
}
fn ensure<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(v) = s
        .get_slot::<PresentationRequestStore>()
        .and_then(|x| x.constructor.get(crate::webidl::realm_id(s)))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c = crate::webidl::create_function(
        s,
        "PresentationRequest",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_accessor(s, p, "onconnectionavailable", get_handler, set_handler)?;
    crate::webidl::define_method(s, p, "getAvailability", 0, get_availability)?;
    crate::webidl::define_method(s, p, "reconnect", 1, reconnect)?;
    crate::webidl::define_method(s, p, "start", 0, start)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let parent = super::event_target::ensure_constructor(s)?;
    crate::webidl::inherit(s, c, parent)?;
    let realm_id = crate::webidl::realm_id(s);
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<PresentationRequestStore>()
        .unwrap()
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
fn construct(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !a.is_construct_call() || a.length() < 1 {
        crate::webidl::throw_type_error(s, "URL required");
        return;
    }
    if let Ok(urls) = v8::Local::<v8::Array>::try_from(a.get(0))
        && urls.length() == 0
    {
        if let Ok(exception) = super::dom_exception::create(
            s,
            "Failed to construct 'PresentationRequest': An empty sequence of URLs is not supported."
                .to_owned(),
            "NotSupportedError".to_owned(),
        ) {
            s.throw_exception(exception.into());
        }
        return;
    }
    super::event_target::attach(s, a.this());
    let url = crate::webidl::value_to_string(s, a.get(0));
    s.get_slot_mut::<PresentationRequestStore>()
        .unwrap()
        .records
        .insert(
            a.this().get_identity_hash().get(),
            RequestRecord { url, handler: None },
        );
    r.set(a.this().into())
}
fn record(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<RequestRecord> {
    s.get_slot::<PresentationRequestStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
fn promise(s: &mut v8::PinScope<'_, '_>, v: v8::Local<'_, v8::Value>, mut r: v8::ReturnValue<'_>) {
    if let Ok(p) = super::writable_stream::resolved_promise(s, v) {
        r.set(p.into())
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
        .get_slot_mut::<PresentationRequestStore>()
        .and_then(|x| x.records.get_mut(&a.this().get_identity_hash().get()))
    {
        v.handler = h
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_availability(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_none() {
        crate::webidl::reject_illegal_invocation_promise(
            s,
            "PresentationRequest",
            "getAvailability",
            r,
        );
        return;
    }
    match super::presentation_availability::create(s, true) {
        Ok(v) => promise(s, v.into(), r),
        Err(e) => crate::webidl::throw_type_error(s, &e),
    }
}
fn connection(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
    id: Option<String>,
    method_name: &str,
) {
    let Some(v) = record(s, a.this()) else {
        crate::webidl::reject_illegal_invocation_promise(s, "PresentationRequest", method_name, r);
        return;
    };
    let id = id.unwrap_or_else(|| {
        let store = s.get_slot_mut::<PresentationRequestStore>().unwrap();
        store.next += 1;
        format!("presentation-{}", store.next)
    });
    match super::presentation_connection::create(s, id, v.url) {
        Ok(v) => promise(s, v.into(), r),
        Err(e) => crate::webidl::throw_type_error(s, &e),
    }
}
fn reconnect(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_none() {
        crate::webidl::reject_illegal_invocation_promise(s, "PresentationRequest", "reconnect", r);
        return;
    }
    let id = crate::webidl::value_to_string(s, a.get(0));
    connection(s, a, r, Some(id), "reconnect")
}
fn start(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    connection(s, a, r, None, "start")
}
