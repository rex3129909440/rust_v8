use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct HtmlSourceElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, SourceRecord>,
}
#[derive(Clone, Default)]
pub(crate) struct SourceRecord {
    pub(crate) strings: HashMap<String, String>,
    pub(crate) width: u32,
    pub(crate) height: u32,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(HtmlSourceElementStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "HTMLSourceElement", c.into())
}
pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(c) = scope
        .get_slot::<HtmlSourceElementStore>()
        .and_then(|s| s.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &c));
    }
    let parent = super::html_element::ensure_constructor(scope)?;
    let c = crate::webidl::create_function(
        scope,
        "HTMLSourceElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    crate::webidl::inherit(scope, c, parent)?;
    let p = crate::webidl::prototype(scope, c)?;
    crate::webidl::reset_constructor_order(scope, p)?;
    super::html_source_element_src_property::define(scope, p)?;
    super::html_source_element_type_property::define(scope, p)?;
    super::html_source_element_srcset_property::define(scope, p)?;
    super::html_source_element_sizes_property::define(scope, p)?;
    super::html_source_element_media_property::define(scope, p)?;
    super::html_source_element_width_property::define(scope, p)?;
    super::html_source_element_height_property::define(scope, p)?;
    crate::webidl::finish_constructor(scope, p, c)?;
    let stored = v8::Global::new(scope, c);
    scope
        .get_slot_mut::<HtmlSourceElementStore>()
        .ok_or_else(|| "HTMLSourceElement state was not prepared".to_owned())?
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
        return Err("cannot create HTMLSourceElement".to_owned());
    }
    super::html_element::attach(scope, o, "SOURCE");
    scope
        .get_slot_mut::<HtmlSourceElementStore>()
        .ok_or_else(|| "HTMLSourceElement state was not prepared".to_owned())?
        .records
        .insert(o.get_identity_hash().get(), SourceRecord::default());
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
) -> Option<SourceRecord> {
    scope
        .get_slot::<HtmlSourceElementStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
pub(crate) fn get_width(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(x) = record(scope, a.this()) {
        r.set(v8::Integer::new_from_unsigned(scope, x.width).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
pub(crate) fn set_width(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = a.get(0).uint32_value(scope).unwrap_or(0);
    if let Some(x) = scope
        .get_slot_mut::<HtmlSourceElementStore>()
        .and_then(|s| s.records.get_mut(&a.this().get_identity_hash().get()))
    {
        x.width = v
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
pub(crate) fn get_height(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(x) = record(scope, a.this()) {
        r.set(v8::Integer::new_from_unsigned(scope, x.height).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
pub(crate) fn set_height(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = a.get(0).uint32_value(scope).unwrap_or(0);
    if let Some(x) = scope
        .get_slot_mut::<HtmlSourceElementStore>()
        .and_then(|s| s.records.get_mut(&a.this().get_identity_hash().get()))
    {
        x.height = v
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
