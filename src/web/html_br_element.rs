use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct HtmlBrElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) clear: HashMap<i32, String>,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(HtmlBrElementStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "HTMLBRElement", c.into())
}
pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(c) = scope
        .get_slot::<HtmlBrElementStore>()
        .and_then(|s| s.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &c));
    }
    let parent = super::html_element::ensure_constructor(scope)?;
    let c = crate::webidl::create_function(
        scope,
        "HTMLBRElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    crate::webidl::inherit(scope, c, parent)?;
    let p = crate::webidl::prototype(scope, c)?;
    crate::webidl::reset_constructor_order(scope, p)?;
    super::html_br_element_clear_property::define(scope, p)?;
    crate::webidl::finish_constructor(scope, p, c)?;
    let stored = v8::Global::new(scope, c);
    scope
        .get_slot_mut::<HtmlBrElementStore>()
        .ok_or_else(|| "HTMLBRElement state was not prepared".to_owned())?
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
        return Err("cannot create HTMLBRElement".to_owned());
    }
    super::html_element::attach(scope, o, "BR");
    scope
        .get_slot_mut::<HtmlBrElementStore>()
        .ok_or_else(|| "HTMLBRElement state was not prepared".to_owned())?
        .clear
        .insert(o.get_identity_hash().get(), String::new());
    Ok(o)
}
pub(crate) fn illegal_constructor(
    s: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(s, "Illegal constructor")
}
pub(crate) fn get_clear(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(x) = s
        .get_slot::<HtmlBrElementStore>()
        .and_then(|q| q.clear.get(&a.this().get_identity_hash().get()))
    {
        if let Some(v) = v8::String::new(s, x) {
            r.set(v.into())
        }
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn set_clear(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = crate::webidl::value_to_string(s, a.get(0));
    if let Some(x) = s
        .get_slot_mut::<HtmlBrElementStore>()
        .and_then(|q| q.clear.get_mut(&a.this().get_identity_hash().get()))
    {
        *x = v
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
