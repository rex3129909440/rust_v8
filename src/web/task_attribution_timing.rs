use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct TaskAttributionTimingStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, Record>,
}
#[derive(Clone)]
struct Record {
    container_type: String,
    container_src: String,
    container_id: String,
    container_name: String,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(TaskAttributionTimingStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "TaskAttributionTiming", c.into())
}
fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<TaskAttributionTimingStore>()
        .and_then(|s| s.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(e) = existing {
        return Ok(v8::Local::new(scope, &e));
    }
    let c = crate::webidl::create_function(
        scope,
        "TaskAttributionTiming",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let p = crate::webidl::prototype(scope, c)?;
    crate::webidl::reset_constructor_order(scope, p)?;
    crate::webidl::define_readonly_accessor(scope, p, "containerType", get_container_type)?;
    crate::webidl::define_readonly_accessor(scope, p, "containerSrc", get_container_src)?;
    crate::webidl::define_readonly_accessor(scope, p, "containerId", get_container_id)?;
    crate::webidl::define_readonly_accessor(scope, p, "containerName", get_container_name)?;
    crate::webidl::define_method(scope, p, "toJSON", 0, to_json)?;
    crate::webidl::finish_constructor(scope, p, c)?;
    let parent = super::performance_entry::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, c, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, c);
    scope
        .get_slot_mut::<TaskAttributionTimingStore>()
        .ok_or_else(|| "TaskAttributionTiming state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'TaskAttributionTiming': Illegal constructor",
    );
}
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    container_type: String,
    container_src: String,
    container_id: String,
    container_name: String,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure_constructor(scope)?;
    let p = crate::webidl::prototype(scope, c)?;
    let o = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, o, p.into()) != Some(true) {
        return Err("cannot create TaskAttributionTiming".to_owned());
    }
    super::performance_entry::attach(
        scope,
        o,
        "unknown".to_owned(),
        "taskattribution".to_owned(),
        0.0,
        0.0,
    );
    scope
        .get_slot_mut::<TaskAttributionTimingStore>()
        .ok_or_else(|| "TaskAttributionTiming state was not prepared".to_owned())?
        .records
        .insert(
            o.get_identity_hash().get(),
            Record {
                container_type,
                container_src,
                container_id,
                container_name,
            },
        );
    Ok(o)
}
fn record(scope: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<Record> {
    scope
        .get_slot::<TaskAttributionTimingStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
fn return_string(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
    select: impl FnOnce(&Record) -> &str,
) {
    let Some(v) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(s) = v8::String::new(scope, select(&v)) {
        r.set(s.into())
    }
}
fn get_container_type(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |v| &v.container_type)
}
fn get_container_src(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |v| &v.container_src)
}
fn get_container_id(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |v| &v.container_id)
}
fn get_container_name(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |v| &v.container_name)
}
fn to_json(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(v) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let o = v8::Object::new(scope);
    define(scope, o, "containerType", &v.container_type);
    define(scope, o, "containerSrc", &v.container_src);
    define(scope, o, "containerId", &v.container_id);
    define(scope, o, "containerName", &v.container_name);
    r.set(o.into())
}
fn define(scope: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>, n: &str, v: &str) {
    if let (Some(k), Some(v)) = (v8::String::new(scope, n), v8::String::new(scope, v)) {
        let _ = o.create_data_property(scope, k.into(), v.into());
    }
}
