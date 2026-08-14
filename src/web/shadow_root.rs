use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct ShadowRootStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, Record>,
}
#[derive(Clone)]
pub(crate) struct Record {
    pub mode: String,
    pub host: v8::Global<v8::Object>,
    pub onslotchange: Option<v8::Global<v8::Value>>,
    pub delegates_focus: bool,
    pub slot_assignment: String,
    pub serializable: bool,
    pub clonable: bool,
    pub adopted: Vec<v8::Global<v8::Object>>,
    pub registry: Option<v8::Global<v8::Object>>,
    pub registry_is_null: bool,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(ShadowRootStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "ShadowRoot", c.into())
}
fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<ShadowRootStore>()
        .and_then(|s| s.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(e) = existing {
        return Ok(v8::Local::new(scope, &e));
    }
    let c = crate::webidl::create_function(
        scope,
        "ShadowRoot",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let p = crate::webidl::prototype(scope, c)?;
    crate::webidl::reset_constructor_order(scope, p)?;
    super::shadow_root_mode_property::define(scope, p)?;
    super::shadow_root_host_property::define(scope, p)?;
    super::shadow_root_onslotchange_property::define(scope, p)?;
    super::shadow_root_inner_html_property::define(scope, p)?;
    super::shadow_root_delegates_focus_property::define(scope, p)?;
    super::shadow_root_slot_assignment_property::define(scope, p)?;
    super::shadow_root_serializable_property::define(scope, p)?;
    super::shadow_root_clonable_property::define(scope, p)?;
    super::shadow_root_active_element_property::define(scope, p)?;
    super::shadow_root_style_sheets_property::define(scope, p)?;
    super::shadow_root_pointer_lock_element_property::define(scope, p)?;
    super::shadow_root_fullscreen_element_property::define(scope, p)?;
    super::shadow_root_adopted_style_sheets_property::define(scope, p)?;
    super::shadow_root_picture_in_picture_element_property::define(scope, p)?;
    super::shadow_root_element_from_point::define(scope, p)?;
    super::shadow_root_elements_from_point::define(scope, p)?;
    super::shadow_root_get_animations::define(scope, p)?;
    super::shadow_root_get_html::define(scope, p)?;
    super::shadow_root_get_selection::define(scope, p)?;
    super::shadow_root_set_html_unsafe::define(scope, p)?;
    super::shadow_root_custom_element_registry_property::define(scope, p)?;
    super::shadow_root_set_html::define(scope, p)?;
    crate::webidl::finish_constructor(scope, p, c)?;
    let parent = super::document_fragment::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, c, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, c);
    scope
        .get_slot_mut::<ShadowRootStore>()
        .ok_or_else(|| "ShadowRoot state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    host: v8::Local<'_, v8::Object>,
    mode: String,
    delegates_focus: bool,
    slot_assignment: String,
    serializable: bool,
    clonable: bool,
    registry: Option<v8::Local<'_, v8::Object>>,
    registry_is_null: bool,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure_constructor(scope)?;
    let p = crate::webidl::prototype(scope, c)?;
    let o = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, o, p.into()) != Some(true) {
        return Err("cannot create ShadowRoot".to_owned());
    }
    super::document_fragment::attach(scope, o);
    if let Some(document) = super::node::record(scope, host)
        .and_then(|record| record.owner_document)
        .map(|document| v8::Local::new(scope, &document))
    {
        super::node::set_owner_document(scope, o, document);
    }
    let record = Record {
        mode,
        host: v8::Global::new(scope, host),
        onslotchange: None,
        delegates_focus,
        slot_assignment,
        serializable,
        clonable,
        adopted: Vec::new(),
        registry: registry.map(|registry| v8::Global::new(scope, registry)),
        registry_is_null,
    };
    scope
        .get_slot_mut::<ShadowRootStore>()
        .ok_or_else(|| "ShadowRoot state was not prepared".to_owned())?
        .records
        .insert(o.get_identity_hash().get(), record);
    Ok(o)
}
fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'ShadowRoot': Illegal constructor",
    );
}
pub(crate) fn record(scope: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<Record> {
    scope
        .get_slot::<ShadowRootStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}

pub(crate) fn host<'s>(
    scope: &v8::PinScope<'s, '_>,
    root: v8::Local<'_, v8::Object>,
) -> Option<v8::Local<'s, v8::Object>> {
    record(scope, root).map(|record| v8::Local::new(scope, &record.host))
}

pub(crate) fn is_closed(scope: &v8::PinScope<'_, '_>, root: v8::Local<'_, v8::Object>) -> bool {
    record(scope, root).is_some_and(|record| record.mode == "closed")
}

pub(crate) fn uses_manual_slot_assignment(
    scope: &v8::PinScope<'_, '_>,
    root: v8::Local<'_, v8::Object>,
) -> bool {
    record(scope, root).is_some_and(|record| record.slot_assignment == "manual")
}

pub(crate) fn update(
    scope: &mut v8::PinScope<'_, '_>,
    o: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut Record),
) {
    if let Some(v) = scope
        .get_slot_mut::<ShadowRootStore>()
        .and_then(|s| s.records.get_mut(&o.get_identity_hash().get()))
    {
        change(v)
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
pub(crate) fn string_get(
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
pub(crate) fn bool_get(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
    select: impl FnOnce(&Record) -> bool,
) {
    if let Some(v) = record(scope, a.this()) {
        r.set(v8::Boolean::new(scope, select(&v)).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
pub(crate) fn document_scoped_element<'s>(
    scope: &v8::PinScope<'s, '_>,
    root: v8::Local<'s, v8::Object>,
    property: &str,
) -> Option<v8::Local<'s, v8::Object>> {
    let document = super::node::record(scope, root)?.owner_document?;
    let document = v8::Local::new(scope, &document);
    let value = super::document::stored_value(scope, document, property)?;
    let element = v8::Local::<v8::Object>::try_from(v8::Local::new(scope, &value)).ok()?;
    let mut current = Some(element);
    while let Some(node) = current {
        if node.strict_equals(root.into()) {
            return Some(element);
        }
        current = super::node::parent(scope, node);
    }
    None
}
