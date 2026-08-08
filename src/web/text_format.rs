use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct TextFormatStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, TextFormatRecord>,
}

#[derive(Clone)]
struct TextFormatRecord {
    range_start: u32,
    range_end: u32,
    underline_style: String,
    underline_thickness: String,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(TextFormatStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "TextFormat", c.into())
}
fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<TextFormatStore>()
        .and_then(|s| s.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(e) = existing {
        return Ok(v8::Local::new(scope, &e));
    }
    let c = crate::webidl::create_function(
        scope,
        "TextFormat",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let p = crate::webidl::prototype(scope, c)?;
    crate::webidl::reset_constructor_order(scope, p)?;
    crate::webidl::define_readonly_accessor(scope, p, "rangeStart", get_range_start)?;
    crate::webidl::define_readonly_accessor(scope, p, "rangeEnd", get_range_end)?;
    crate::webidl::define_readonly_accessor(scope, p, "underlineStyle", get_underline_style)?;
    crate::webidl::define_readonly_accessor(
        scope,
        p,
        "underlineThickness",
        get_underline_thickness,
    )?;
    crate::webidl::finish_constructor(scope, p, c)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, c);
    scope
        .get_slot_mut::<TextFormatStore>()
        .ok_or_else(|| "TextFormat state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(scope, "Failed to construct 'TextFormat': use new");
        return;
    }
    let init = v8::Local::<v8::Object>::try_from(arguments.get(0)).ok();
    let record = TextFormatRecord {
        range_start: init.map(|v| uint(scope, v, "rangeStart")).unwrap_or(0),
        range_end: init.map(|v| uint(scope, v, "rangeEnd")).unwrap_or(0),
        underline_style: init
            .and_then(|v| string(scope, v, "underlineStyle"))
            .unwrap_or_default(),
        underline_thickness: init
            .and_then(|v| string(scope, v, "underlineThickness"))
            .unwrap_or_default(),
    };
    let object = arguments.this();
    scope
        .get_slot_mut::<TextFormatStore>()
        .expect("TextFormat state")
        .records
        .insert(object.get_identity_hash().get(), record);
    result.set(object.into());
}
fn uint(scope: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>, n: &str) -> u32 {
    v8::String::new(scope, n)
        .and_then(|k| o.get(scope, k.into()))
        .filter(|v| !v.is_undefined())
        .and_then(|v| v.uint32_value(scope))
        .unwrap_or(0)
}
fn string(scope: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>, n: &str) -> Option<String> {
    let k = v8::String::new(scope, n)?;
    let v = o.get(scope, k.into())?;
    if v.is_undefined() {
        None
    } else {
        Some(crate::webidl::value_to_string(scope, v))
    }
}
fn record(scope: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<TextFormatRecord> {
    scope
        .get_slot::<TextFormatStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
pub(crate) fn is_instance(scope: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> bool {
    record(scope, o).is_some()
}
fn return_uint(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
    select: impl FnOnce(&TextFormatRecord) -> u32,
) {
    if let Some(v) = record(scope, a.this()) {
        r.set(v8::Integer::new_from_unsigned(scope, select(&v)).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn return_string(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
    select: impl FnOnce(&TextFormatRecord) -> &str,
) {
    let Some(v) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(s) = v8::String::new(scope, select(&v)) {
        r.set(s.into())
    }
}
fn get_range_start(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_uint(s, a, r, |v| v.range_start)
}
fn get_range_end(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_uint(s, a, r, |v| v.range_end)
}
fn get_underline_style(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |v| &v.underline_style)
}
fn get_underline_thickness(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |v| &v.underline_thickness)
}
