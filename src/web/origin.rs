use std::collections::HashMap;

#[derive(Clone)]
struct OriginRecord {
    scheme: String,
    host: String,
    port: Option<u16>,
    opaque: bool,
}
#[derive(Default)]
pub(crate) struct OriginStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, OriginRecord>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(OriginStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(s)?;
    crate::webidl::define_global(s, "Origin", c.into())
}
fn ensure_constructor<'s>(
    s: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(s);
    if let Some(c) = s
        .get_slot::<OriginStore>()
        .and_then(|x| x.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(s, &c));
    }
    let c =
        crate::webidl::create_function(s, "Origin", 0, v8::ConstructorBehavior::Allow, construct)?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_readonly_accessor(s, p, "opaque", get_opaque)?;
    crate::webidl::define_method(s, p, "isSameOrigin", 1, is_same_origin)?;
    crate::webidl::define_method(s, p, "isSameSite", 1, is_same_site)?;
    crate::webidl::finish_constructor(s, p, c)?;
    crate::webidl::define_method(s, c.into(), "from", 1, from)?;
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<OriginStore>()
        .ok_or_else(|| "Origin state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
fn construct(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !a.is_construct_call() {
        crate::webidl::throw_type_error(s, "Origin must be constructed with new");
        return;
    }
    let value = if a.length() == 0 {
        opaque()
    } else {
        parse(&crate::webidl::value_to_string(s, a.get(0)))
    };
    s.get_slot_mut::<OriginStore>()
        .expect("Origin state")
        .records
        .insert(a.this().get_identity_hash().get(), value);
    r.set(a.this().into())
}
fn create<'s>(
    s: &mut v8::PinScope<'s, '_>,
    value: OriginRecord,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure_constructor(s)?;
    let p = crate::webidl::prototype(s, c)?;
    let o = v8::Object::new(s);
    if crate::webidl::set_platform_prototype(s, o, p.into()) != Some(true) {
        return Err("cannot create Origin".to_owned());
    }
    s.get_slot_mut::<OriginStore>()
        .ok_or_else(|| "Origin state was not prepared".to_owned())?
        .records
        .insert(o.get_identity_hash().get(), value);
    Ok(o)
}
fn record(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<OriginRecord> {
    s.get_slot::<OriginStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
fn opaque() -> OriginRecord {
    OriginRecord {
        scheme: String::new(),
        host: String::new(),
        port: None,
        opaque: true,
    }
}
fn parse(value: &str) -> OriginRecord {
    let Ok(url) = url::Url::parse(value) else {
        return opaque();
    };
    let Some(host) = url.host_str() else {
        return opaque();
    };
    if matches!(url.scheme(), "data" | "about" | "javascript") {
        return opaque();
    }
    OriginRecord {
        scheme: url.scheme().to_owned(),
        host: host.to_ascii_lowercase(),
        port: url.port_or_known_default(),
        opaque: false,
    }
}
fn from(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if a.length() < 1 {
        crate::webidl::throw_type_error(
            s,
            "Failed to execute 'from' on 'Origin': 1 argument required, but only 0 present.",
        );
        return;
    }
    if let Ok(o) = v8::Local::<v8::Object>::try_from(a.get(0)) {
        if let Some(value) = record(s, o) {
            if let Ok(value) = create(s, value) {
                r.set(value.into())
            }
            return;
        }
    }
    let value = parse(&crate::webidl::value_to_string(s, a.get(0)));
    if let Ok(value) = create(s, value) {
        r.set(value.into())
    }
}
fn get_opaque(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(x) = record(s, a.this()) {
        r.set(v8::Boolean::new(s, x.opaque).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn other(
    s: &mut v8::PinScope<'_, '_>,
    a: &v8::FunctionCallbackArguments<'_>,
    method: &str,
) -> Option<(OriginRecord, OriginRecord)> {
    let Some(this) = record(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return None;
    };
    let Ok(o) = v8::Local::<v8::Object>::try_from(a.get(0)) else {
        crate::webidl::throw_type_error(s, &format!("{method} requires an Origin"));
        return None;
    };
    let Some(other) = record(s, o) else {
        crate::webidl::throw_type_error(s, &format!("{method} requires an Origin"));
        return None;
    };
    Some((this, other))
}
fn is_same_origin(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some((x, y)) = other(s, &a, "isSameOrigin") else {
        return;
    };
    let same =
        !x.opaque && !y.opaque && x.scheme == y.scheme && x.host == y.host && x.port == y.port;
    r.set(v8::Boolean::new(s, same).into())
}
fn is_same_site(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some((x, y)) = other(s, &a, "isSameSite") else {
        return;
    };
    let same = !x.opaque && !y.opaque && site(&x.host) == site(&y.host);
    r.set(v8::Boolean::new(s, same).into())
}
fn site(host: &str) -> String {
    let mut p = host.rsplit('.');
    let last = p.next().unwrap_or("");
    let second = p.next().unwrap_or("");
    if second.is_empty() {
        last.to_owned()
    } else {
        format!("{second}.{last}")
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<OriginStore>() {
        store.constructor.remove(realm_id);
    }
}
