use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct HtmlHtmlElementStore {
    pub(crate) constructors: HashMap<i32, v8::Global<v8::Function>>,
    pub(crate) versions: HashMap<i32, String>,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(HtmlHtmlElementStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "HTMLHtmlElement", constructor.into())
}
pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<HtmlHtmlElementStore>()
        .and_then(|store| store.constructors.get(&crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let parent = super::html_element::ensure_constructor(scope)?;
    let constructor = crate::webidl::create_function(
        scope,
        "HTMLHtmlElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::html_html_element_version_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<HtmlHtmlElementStore>()
        .ok_or_else(|| "HTMLHtmlElement state was not prepared".to_owned())?
        .constructors
        .insert(realm_id, stored);
    Ok(constructor)
}
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create HTMLHtmlElement".to_owned());
    }
    super::html_element::attach(scope, object, "HTML");
    scope
        .get_slot_mut::<HtmlHtmlElementStore>()
        .ok_or_else(|| "HTMLHtmlElement state was not prepared".to_owned())?
        .versions
        .insert(object.get_identity_hash().get(), String::new());
    Ok(object)
}
pub(crate) fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor");
}
pub(crate) fn get_version(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(value) = scope
        .get_slot::<HtmlHtmlElementStore>()
        .and_then(|store| store.versions.get(&a.this().get_identity_hash().get()))
    {
        if let Some(value) = v8::String::new(scope, value) {
            r.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn set_version(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, a.get(0));
    if let Some(current) = scope
        .get_slot_mut::<HtmlHtmlElementStore>()
        .and_then(|store| store.versions.get_mut(&a.this().get_identity_hash().get()))
    {
        *current = value;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
