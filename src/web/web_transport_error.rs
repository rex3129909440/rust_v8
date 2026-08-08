use std::collections::HashMap;
#[derive(Clone)]
struct ErrorData {
    code: Option<u32>,
    source: String,
}
#[derive(Default)]
pub(crate) struct WebTransportErrorStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, ErrorData>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(WebTransportErrorStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "WebTransportError", c.into())
}
fn ensure<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(s);
    if let Some(v) = s
        .get_slot::<WebTransportErrorStore>()
        .and_then(|x| x.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c = crate::webidl::create_function(
        s,
        "WebTransportError",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_readonly_accessor(s, p, "streamErrorCode", code)?;
    crate::webidl::define_readonly_accessor(s, p, "source", source)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let parent = super::dom_exception::ensure_constructor(s)?;
    crate::webidl::inherit(s, c, parent)?;
    let g = v8::Global::new(s, c);
    s.get_slot_mut::<WebTransportErrorStore>()
        .ok_or_else(|| "WebTransportError state missing".to_owned())?
        .constructor
        .insert(realm_id, g);
    Ok(c)
}
fn construct(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !a.is_construct_call() {
        crate::webidl::throw_type_error(s, "Please use the 'new' operator");
        return;
    }
    let init = v8::Local::<v8::Object>::try_from(a.get(0)).ok();
    let get = |n| init.and_then(|o| v8::String::new(s, n).and_then(|k| o.get(s, k.into())));
    let message = get("message")
        .map(|v| crate::webidl::value_to_string(s, v))
        .unwrap_or_default();
    let source = get("source")
        .map(|v| crate::webidl::value_to_string(s, v))
        .unwrap_or_else(|| "stream".to_owned());
    let code = get("streamErrorCode").and_then(|v| v.uint32_value(s));
    super::dom_exception::attach(s, a.this(), "NetworkError".to_owned(), message, 0);
    s.get_slot_mut::<WebTransportErrorStore>()
        .unwrap()
        .records
        .insert(
            a.this().get_identity_hash().get(),
            ErrorData { code, source },
        );
    r.set(a.this().into())
}
fn record(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<ErrorData> {
    s.get_slot::<WebTransportErrorStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
fn code(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        if let Some(v) = v.code {
            r.set(v8::Integer::new_from_unsigned(s, v).into())
        } else {
            r.set(v8::null(s).into())
        }
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn source(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this())
        && let Some(v) = v8::String::new(s, &v.source)
    {
        r.set(v.into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<WebTransportErrorStore>() {
        store.constructor.remove(realm_id);
    }
}
