#[derive(Default)]
pub(crate) struct NodeFilterStore {
    namespace: crate::webidl::RealmConstructor,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(NodeFilterStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let namespace = ensure_namespace(scope)?;
    crate::webidl::define_global(scope, "NodeFilter", namespace.into())
}

fn ensure_namespace<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<NodeFilterStore>()
        .and_then(|store| store.namespace.get(realm_id))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let namespace = crate::webidl::create_function(
        scope,
        "NodeFilter",
        0,
        v8::ConstructorBehavior::Throw,
        illegal_invocation,
    )?;
    define_u32(scope, namespace.into(), "FILTER_ACCEPT", 1)?;
    define_u32(scope, namespace.into(), "FILTER_REJECT", 2)?;
    define_u32(scope, namespace.into(), "FILTER_SKIP", 3)?;
    define_u32(scope, namespace.into(), "SHOW_ALL", u32::MAX)?;
    define_u32(scope, namespace.into(), "SHOW_ELEMENT", 1)?;
    define_u32(scope, namespace.into(), "SHOW_ATTRIBUTE", 2)?;
    define_u32(scope, namespace.into(), "SHOW_TEXT", 4)?;
    define_u32(scope, namespace.into(), "SHOW_CDATA_SECTION", 8)?;
    define_u32(scope, namespace.into(), "SHOW_ENTITY_REFERENCE", 16)?;
    define_u32(scope, namespace.into(), "SHOW_ENTITY", 32)?;
    define_u32(scope, namespace.into(), "SHOW_PROCESSING_INSTRUCTION", 64)?;
    define_u32(scope, namespace.into(), "SHOW_COMMENT", 128)?;
    define_u32(scope, namespace.into(), "SHOW_DOCUMENT", 256)?;
    define_u32(scope, namespace.into(), "SHOW_DOCUMENT_TYPE", 512)?;
    define_u32(scope, namespace.into(), "SHOW_DOCUMENT_FRAGMENT", 1024)?;
    define_u32(scope, namespace.into(), "SHOW_NOTATION", 2048)?;
    let stored = v8::Global::new(scope, namespace);
    scope
        .get_slot_mut::<NodeFilterStore>()
        .ok_or_else(|| "NodeFilter state was not prepared".to_owned())?
        .namespace
        .insert(realm_id, stored);
    Ok(namespace)
}

fn define_u32(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: u32,
) -> Result<(), String> {
    let key = crate::webidl::string(scope, name)?;
    let value = v8::Number::new(scope, value as f64);
    if object.define_own_property(
        scope,
        key.into(),
        value.into(),
        v8::PropertyAttribute::READ_ONLY | v8::PropertyAttribute::DONT_DELETE,
    ) == Some(true)
    {
        Ok(())
    } else {
        Err(format!("cannot define NodeFilter.{name}"))
    }
}

fn illegal_invocation(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor");
}
