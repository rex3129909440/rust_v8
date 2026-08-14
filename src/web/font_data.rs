use std::collections::HashMap;
#[derive(Clone)]
struct FontRecord {
    postscript: String,
    full: String,
    family: String,
    style: String,
    bytes: Vec<u8>,
}
#[derive(Default)]
pub(crate) struct FontDataStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, FontRecord>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(FontDataStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "FontData", c.into())
}
fn ensure<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(v) = s
        .get_slot::<FontDataStore>()
        .and_then(|x| x.constructor.get(crate::webidl::realm_id(s)))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c =
        crate::webidl::create_function(s, "FontData", 0, v8::ConstructorBehavior::Allow, illegal)?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_readonly_accessor(s, p, "postscriptName", postscript_name)?;
    crate::webidl::define_readonly_accessor(s, p, "fullName", full_name)?;
    crate::webidl::define_readonly_accessor(s, p, "family", family)?;
    crate::webidl::define_readonly_accessor(s, p, "style", style)?;
    crate::webidl::define_method(s, p, "blob", 0, blob)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let realm_id = crate::webidl::realm_id(s);
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<FontDataStore>()
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
    postscript: String,
    full: String,
    family: String,
    style: String,
    bytes: Vec<u8>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure(s)?;
    let p = crate::webidl::prototype(s, c)?;
    let o = v8::Object::new(s);
    if crate::webidl::set_platform_prototype(s, o, p.into()) != Some(true) {
        return Err("cannot create FontData".to_owned());
    }
    s.get_slot_mut::<FontDataStore>().unwrap().records.insert(
        o.get_identity_hash().get(),
        FontRecord {
            postscript,
            full,
            family,
            style,
            bytes,
        },
    );
    Ok(o)
}
fn record(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<FontRecord> {
    s.get_slot::<FontDataStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
fn text(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
    f: impl FnOnce(FontRecord) -> String,
) {
    if let Some(v) = record(s, a.this())
        && let Some(x) = v8::String::new(s, &f(v))
    {
        r.set(x.into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn postscript_name(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    text(s, a, r, |v| v.postscript)
}
fn full_name(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    text(s, a, r, |v| v.full)
}
fn family(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    text(s, a, r, |v| v.family)
}
fn style(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    text(s, a, r, |v| v.style)
}
fn blob(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(v) = record(s, a.this()) else {
        crate::webidl::reject_illegal_invocation_promise(s, "FontData", "blob", r);
        return;
    };
    if v.bytes.is_empty() {
        if let Some(promise) = crate::webidl::rejected_type_error_promise(
            s,
            &format!("Font data for {} could not be accessed.", v.postscript),
        ) {
            r.set(promise.into());
        }
        return;
    }
    // Chromium returns the raw SFNT/TTC stream and deliberately does not
    // infer a format-specific MIME type from the installed font extension.
    match super::blob::create(s, v.bytes, "application/octet-stream") {
        Ok(x) => {
            if let Ok(p) = super::writable_stream::resolved_promise(s, x.into()) {
                r.set(p.into())
            }
        }
        Err(e) => crate::webidl::throw_type_error(s, &e),
    }
}
