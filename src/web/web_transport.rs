use std::collections::HashMap;
#[derive(Clone)]
struct Transport {
    incoming_uni: v8::Global<v8::Object>,
    incoming_bidi: v8::Global<v8::Object>,
    datagrams: v8::Global<v8::Object>,
    ready: v8::Global<v8::Promise>,
    closed: v8::Global<v8::Promise>,
    close_resolver: v8::Global<v8::PromiseResolver>,
    protocol: String,
    active: bool,
}
#[derive(Default)]
pub(crate) struct WebTransportStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, Transport>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(WebTransportStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "WebTransport", c.into())
}
fn ensure<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(s);
    if let Some(v) = s
        .get_slot::<WebTransportStore>()
        .and_then(|x| x.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c = crate::webidl::create_function(
        s,
        "WebTransport",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_readonly_accessor(s, p, "incomingUnidirectionalStreams", incoming_uni)?;
    crate::webidl::define_readonly_accessor(s, p, "incomingBidirectionalStreams", incoming_bidi)?;
    crate::webidl::define_readonly_accessor(s, p, "datagrams", datagrams)?;
    crate::webidl::define_readonly_accessor(s, p, "ready", ready)?;
    crate::webidl::define_readonly_accessor(s, p, "closed", closed)?;
    crate::webidl::define_method(s, p, "close", 0, close)?;
    crate::webidl::define_method(s, p, "createBidirectionalStream", 0, create_bidi)?;
    crate::webidl::define_method(s, p, "createUnidirectionalStream", 0, create_uni)?;
    crate::webidl::define_readonly_accessor(s, p, "protocol", protocol)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let g = v8::Global::new(s, c);
    s.get_slot_mut::<WebTransportStore>()
        .ok_or_else(|| "WebTransport state missing".to_owned())?
        .constructor
        .insert(realm_id, g);
    Ok(c)
}
fn construct(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !a.is_construct_call() || a.length() < 1 {
        crate::webidl::throw_type_error(s, "WebTransport requires a URL");
        return;
    }
    let Some(url) = crate::webidl::dom_string(s, a.get(0)) else {
        return;
    };
    if !url.starts_with("https://") {
        if let Ok(exception) = super::dom_exception::create(
            s,
            format!("Failed to construct 'WebTransport': The URL '{url}' is invalid."),
            "SyntaxError".to_owned(),
        ) {
            s.throw_exception(exception.into());
        }
        return;
    }
    let incoming_uni = match super::readable_stream::create_empty(s) {
        Ok(v) => v,
        Err(_) => return,
    };
    let incoming_bidi = match super::readable_stream::create_empty(s) {
        Ok(v) => v,
        Err(_) => return,
    };
    let datagrams = match super::web_transport_datagram_duplex_stream::create(s) {
        Ok(v) => v,
        Err(_) => return,
    };
    let ready = match super::writable_stream::resolved_promise(s, v8::undefined(s).into()) {
        Ok(v) => v,
        Err(_) => return,
    };
    let resolver = match v8::PromiseResolver::new(s) {
        Some(v) => v,
        None => return,
    };
    let closed = resolver.get_promise(s);
    let t = Transport {
        incoming_uni: v8::Global::new(s, incoming_uni),
        incoming_bidi: v8::Global::new(s, incoming_bidi),
        datagrams: v8::Global::new(s, datagrams),
        ready: v8::Global::new(s, ready),
        closed: v8::Global::new(s, closed),
        close_resolver: v8::Global::new(s, resolver),
        protocol: String::new(),
        active: true,
    };
    s.get_slot_mut::<WebTransportStore>()
        .unwrap()
        .records
        .insert(a.this().get_identity_hash().get(), t);
    r.set(a.this().into())
}
fn record(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<Transport> {
    s.get_slot::<WebTransportStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
fn object(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
    pick: impl FnOnce(Transport) -> v8::Global<v8::Object>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Local::new(s, &pick(v)).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn promise(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
    property_name: &str,
    pick: impl FnOnce(Transport) -> v8::Global<v8::Promise>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Local::new(s, &pick(v)).into())
    } else {
        let message = format!(
            "Failed to read the '{property_name}' property from 'WebTransport': Illegal invocation"
        );
        if let Some(promise) = crate::webidl::rejected_type_error_promise(s, &message) {
            r.set(promise.into())
        }
    }
}
fn incoming_uni(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    object(s, a, r, |v| v.incoming_uni)
}
fn incoming_bidi(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    object(s, a, r, |v| v.incoming_bidi)
}
fn datagrams(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    object(s, a, r, |v| v.datagrams)
}
fn ready(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    promise(s, a, r, "ready", |v| v.ready)
}
fn closed(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    promise(s, a, r, "closed", |v| v.closed)
}
fn protocol(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this())
        && let Some(v) = v8::String::new(s, &v.protocol)
    {
        r.set(v.into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn create_bidi(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_none() {
        crate::webidl::reject_illegal_invocation_promise(
            s,
            "WebTransport",
            "createBidirectionalStream",
            r,
        );
        return;
    }
    if !record(s, a.this()).is_some_and(|v| v.active) {
        crate::webidl::throw_type_error(s, "WebTransport is closed");
        return;
    }
    if let Ok(v) = super::web_transport_bidirectional_stream::create(s)
        && let Ok(p) = super::writable_stream::resolved_promise(s, v.into())
    {
        r.set(p.into())
    }
}
fn create_uni(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_none() {
        crate::webidl::reject_illegal_invocation_promise(
            s,
            "WebTransport",
            "createUnidirectionalStream",
            r,
        );
        return;
    }
    if !record(s, a.this()).is_some_and(|v| v.active) {
        crate::webidl::throw_type_error(s, "WebTransport is closed");
        return;
    }
    if let Ok(v) = super::writable_stream::create_empty(s)
        && let Ok(p) = super::writable_stream::resolved_promise(s, v.into())
    {
        r.set(p.into())
    }
}
fn close(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let resolver = {
        let Some(v) = s
            .get_slot_mut::<WebTransportStore>()
            .and_then(|x| x.records.get_mut(&a.this().get_identity_hash().get()))
        else {
            crate::webidl::throw_type_error(s, "Illegal invocation");
            return;
        };
        v.active = false;
        v.close_resolver.clone()
    };
    let resolver = v8::Local::new(s, &resolver);
    let _ = resolver.resolve(s, v8::undefined(s).into());
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<WebTransportStore>() {
        store.constructor.remove(realm_id);
    }
}
