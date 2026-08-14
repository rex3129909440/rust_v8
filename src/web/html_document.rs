use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Default)]
pub(crate) struct HtmlDocumentStore {
    constructors: HashMap<i32, v8::Global<v8::Function>>,
    instances: HashSet<i32>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(HtmlDocumentStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "HTMLDocument", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<HtmlDocumentStore>()
        .and_then(|store| store.constructors.get(&crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let parent = super::document::ensure_constructor(scope)?;
    let constructor = crate::webidl::create_function(
        scope,
        "HTMLDocument",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<HtmlDocumentStore>()
        .ok_or_else(|| "HTMLDocument state was not prepared".to_owned())?
        .constructors
        .insert(realm_id, stored);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    create_from_source(scope, String::new())
}

pub(crate) fn create_from_source<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    source: String,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create HTMLDocument".to_owned());
    }
    super::node::attach(scope, object, 9, "#document".to_owned(), None);
    super::document::attach(scope, object, "text/html".to_owned());
    super::document::set_string_value(scope, object, "URL", "about:blank");
    super::document::set_string_value(scope, object, "documentURI", "about:blank");
    super::document::set_string_value(scope, object, "fallbackBaseURL", "about:blank");
    super::document::set_string_value(scope, object, "compatMode", "CSS1Compat");
    if let Some(store) = scope.get_slot_mut::<HtmlDocumentStore>() {
        store.instances.insert(object.get_identity_hash().get());
    }
    if !source.is_empty() {
        super::document::parse_source(scope, object, &source)?;
    }
    Ok(object)
}

pub(crate) fn is_instance(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> bool {
    scope
        .get_slot::<HtmlDocumentStore>()
        .is_some_and(|store| store.instances.contains(&object.get_identity_hash().get()))
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor");
}
