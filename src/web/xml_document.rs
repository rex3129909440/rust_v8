use std::collections::HashSet;

#[derive(Default)]
pub(crate) struct XmlDocumentStore {
    constructor: crate::webidl::RealmConstructor,
    instances: HashSet<i32>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(XmlDocumentStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "XMLDocument", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<XmlDocumentStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let parent = super::document::ensure_constructor(scope)?;
    let constructor = crate::webidl::create_function(
        scope,
        "XMLDocument",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<XmlDocumentStore>()
        .ok_or_else(|| "XMLDocument state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

#[allow(dead_code)]
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    source: String,
) -> Result<v8::Local<'s, v8::Object>, String> {
    create_with_type(scope, source, "application/xml")
}

pub(crate) fn create_with_type<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    source: String,
    content_type: &str,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create XMLDocument".to_owned());
    }
    super::node::attach(scope, object, 9, "#document".to_owned(), None);
    super::document::attach(scope, object, content_type.to_owned());
    super::document::set_string_value(scope, object, "URL", "about:blank");
    super::document::set_string_value(scope, object, "documentURI", "about:blank");
    super::document::set_string_value(scope, object, "fallbackBaseURL", "about:blank");
    super::document::set_string_value(scope, object, "compatMode", "CSS1Compat");
    if let Some(store) = scope.get_slot_mut::<XmlDocumentStore>() {
        store.instances.insert(object.get_identity_hash().get());
    }
    if !source.is_empty() {
        super::document::parse_source(scope, object, &source)?;
    }
    Ok(object)
}

pub(crate) fn is_instance(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> bool {
    scope
        .get_slot::<XmlDocumentStore>()
        .is_some_and(|store| store.instances.contains(&object.get_identity_hash().get()))
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'XMLDocument': Illegal constructor",
    );
}
