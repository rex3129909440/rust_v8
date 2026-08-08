use std::collections::HashMap;
#[derive(Default)]
pub(crate) struct HtmlProgressElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, ProgressRecord>,
}
#[derive(Clone)]
pub(crate) struct ProgressRecord {
    pub(crate) value: f64,
    pub(crate) max: f64,
    pub(crate) has_value: bool,
    pub(crate) labels: v8::Global<v8::Object>,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(HtmlProgressElementStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "HTMLProgressElement", c.into())
}
pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(c) = scope
        .get_slot::<HtmlProgressElementStore>()
        .and_then(|s| s.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &c));
    }
    let parent = super::html_element::ensure_constructor(scope)?;
    let c = crate::webidl::create_function(
        scope,
        "HTMLProgressElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    crate::webidl::inherit(scope, c, parent)?;
    let p = crate::webidl::prototype(scope, c)?;
    crate::webidl::reset_constructor_order(scope, p)?;
    super::html_progress_element_value_property::define(scope, p)?;
    super::html_progress_element_max_property::define(scope, p)?;
    super::html_progress_element_position_property::define(scope, p)?;
    super::html_progress_element_labels_property::define(scope, p)?;
    crate::webidl::finish_constructor(scope, p, c)?;
    let stored = v8::Global::new(scope, c);
    scope
        .get_slot_mut::<HtmlProgressElementStore>()
        .ok_or_else(|| "HTMLProgressElement state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, stored);
    Ok(c)
}
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure_constructor(scope)?;
    let p = crate::webidl::prototype(scope, c)?;
    let o = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, o, p.into()) != Some(true) {
        return Err("cannot create HTMLProgressElement".to_owned());
    }
    super::html_element::attach(scope, o, "PROGRESS");
    let labels = super::node_list::create(scope, Vec::new())?;
    super::node_list::register_labels_owner(scope, labels, o);
    let labels = v8::Global::new(scope, labels);
    scope
        .get_slot_mut::<HtmlProgressElementStore>()
        .ok_or_else(|| "HTMLProgressElement state was not prepared".to_owned())?
        .records
        .insert(
            o.get_identity_hash().get(),
            ProgressRecord {
                value: 0.0,
                max: 1.0,
                has_value: false,
                labels,
            },
        );
    Ok(o)
}
pub(crate) fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor")
}
pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    o: v8::Local<'_, v8::Object>,
) -> Option<ProgressRecord> {
    scope
        .get_slot::<HtmlProgressElementStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
pub(crate) fn get_value(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(x) = record(scope, a.this()) {
        r.set(v8::Number::new(scope, x.value.min(x.max).max(0.0)).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
pub(crate) fn set_value(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = a.get(0).number_value(scope).unwrap_or(0.0).max(0.0);
    if let Some(x) = scope
        .get_slot_mut::<HtmlProgressElementStore>()
        .and_then(|s| s.records.get_mut(&a.this().get_identity_hash().get()))
    {
        x.value = v.min(x.max);
        x.has_value = true
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
pub(crate) fn get_max(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(x) = record(scope, a.this()) {
        r.set(v8::Number::new(scope, x.max).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
pub(crate) fn set_max(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = a.get(0).number_value(scope).unwrap_or(1.0);
    let v = if v > 0.0 { v } else { 1.0 };
    if let Some(x) = scope
        .get_slot_mut::<HtmlProgressElementStore>()
        .and_then(|s| s.records.get_mut(&a.this().get_identity_hash().get()))
    {
        x.max = v;
        x.value = x.value.min(v)
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
pub(crate) fn get_position(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(x) = record(scope, a.this()) {
        r.set(v8::Number::new(scope, if x.has_value { x.value / x.max } else { -1.0 }).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
pub(crate) fn get_labels(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(x) = record(scope, a.this()) {
        r.set(v8::Local::new(scope, &x.labels).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
