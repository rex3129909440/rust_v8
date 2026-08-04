use std::collections::HashMap;

#[derive(Clone)]
pub(crate) struct DocumentTypeRecord {
    pub name: String,
    pub public_id: String,
    pub system_id: String,
}
#[derive(Default)]
pub(crate) struct DocumentTypeStore {
    constructors: HashMap<i32, v8::Global<v8::Function>>,
    records: HashMap<i32, DocumentTypeRecord>,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(DocumentTypeStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "DocumentType", c.into())
}
fn ensure_constructor<'s>(
    s: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(c) = s
        .get_slot::<DocumentTypeStore>()
        .and_then(|x| x.constructors.get(&crate::webidl::realm_id(s)))
        .cloned()
    {
        return Ok(v8::Local::new(s, &c));
    }
    let c = crate::webidl::create_function(
        s,
        "DocumentType",
        0,
        v8::ConstructorBehavior::Allow,
        illegal,
    )?;
    let parent = super::node::ensure_constructor(s)?;
    crate::webidl::inherit(s, c, parent)?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    super::document_type_name_property::define(s, p)?;
    super::document_type_public_id_property::define(s, p)?;
    super::document_type_system_id_property::define(s, p)?;
    super::document_type_after::define(s, p)?;
    super::document_type_before::define(s, p)?;
    super::document_type_remove::define(s, p)?;
    super::document_type_replace_with::define(s, p)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let unscopables = crate::webidl::new_unscopables(s)?;
    crate::webidl::define_unscopable(s, unscopables, "after")?;
    crate::webidl::define_unscopable(s, unscopables, "before")?;
    crate::webidl::define_unscopable(s, unscopables, "remove")?;
    crate::webidl::define_unscopable(s, unscopables, "replaceWith")?;
    crate::webidl::attach_unscopables(s, p, unscopables)?;
    let realm_id = crate::webidl::realm_id(s);
    let stored = v8::Global::new(s, c);
    s.get_slot_mut::<DocumentTypeStore>()
        .ok_or_else(|| "DocumentType state was not prepared".to_owned())?
        .constructors
        .insert(realm_id, stored);
    Ok(c)
}
pub(crate) fn create<'s>(
    s: &mut v8::PinScope<'s, '_>,
    name: &str,
    public_id: &str,
    system_id: &str,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure_constructor(s)?;
    let p = crate::webidl::prototype(s, c)?;
    let o = v8::Object::new(s);
    if crate::webidl::set_platform_prototype(s, o, p.into()) != Some(true) {
        return Err("cannot create DocumentType".to_owned());
    }
    super::node::attach(s, o, 10, name.to_owned(), None);
    s.get_slot_mut::<DocumentTypeStore>()
        .ok_or_else(|| "DocumentType state was not prepared".to_owned())?
        .records
        .insert(
            o.get_identity_hash().get(),
            DocumentTypeRecord {
                name: name.to_owned(),
                public_id: public_id.to_owned(),
                system_id: system_id.to_owned(),
            },
        );
    Ok(o)
}
fn illegal(
    s: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(s, "Illegal constructor")
}
pub(crate) fn record(
    s: &v8::PinScope<'_, '_>,
    o: v8::Local<'_, v8::Object>,
) -> Option<DocumentTypeRecord> {
    s.get_slot::<DocumentTypeStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
pub(crate) fn serialize(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<String> {
    let value = record(s, o)?;
    if !value.public_id.is_empty() {
        Some(format!(
            "<!DOCTYPE {} PUBLIC \"{}\" \"{}\">",
            value.name, value.public_id, value.system_id
        ))
    } else if !value.system_id.is_empty() {
        Some(format!(
            "<!DOCTYPE {} SYSTEM \"{}\">",
            value.name, value.system_id
        ))
    } else {
        Some(format!("<!DOCTYPE {}>", value.name))
    }
}
