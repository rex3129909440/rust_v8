use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Default)]
pub(crate) struct HtmlTemplateElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, TemplateRecord>,
    template_documents: HashMap<i32, v8::Global<v8::Object>>,
    inert_document_ids: HashSet<i32>,
}

#[derive(Clone)]
pub(crate) struct TemplateRecord {
    pub(crate) content: v8::Global<v8::Object>,
    pub(crate) shadow_root_mode: String,
    pub(crate) shadow_root_delegates_focus: bool,
    pub(crate) shadow_root_clonable: bool,
    pub(crate) shadow_root_serializable: bool,
    pub(crate) shadow_root_custom_element_registry: Option<v8::Global<v8::Object>>,
    pub(crate) html_for: String,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(HtmlTemplateElementStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "HTMLTemplateElement", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<HtmlTemplateElementStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let parent = super::html_element::ensure_constructor(scope)?;
    let constructor = crate::webidl::create_function(
        scope,
        "HTMLTemplateElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::html_template_element_content_property::define(scope, prototype)?;
    super::html_template_element_shadow_root_mode_property::define(scope, prototype)?;
    super::html_template_element_shadow_root_delegates_focus_property::define(scope, prototype)?;
    super::html_template_element_shadow_root_clonable_property::define(scope, prototype)?;
    super::html_template_element_shadow_root_serializable_property::define(scope, prototype)?;
    super::html_template_element_shadow_root_custom_element_registry_property::define(
        scope, prototype,
    )?;
    super::html_template_element_html_for_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<HtmlTemplateElementStore>()
        .ok_or_else(|| "HTMLTemplateElement state was not prepared".to_owned())?
        .constructor
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
        return Err("cannot create HTMLTemplateElement".to_owned());
    }
    super::html_element::attach(scope, object, "TEMPLATE");
    let content = super::document_fragment::create(scope)?;
    let content = v8::Global::new(scope, content);
    scope
        .get_slot_mut::<HtmlTemplateElementStore>()
        .ok_or_else(|| "HTMLTemplateElement state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            TemplateRecord {
                content,
                shadow_root_mode: String::new(),
                shadow_root_delegates_focus: false,
                shadow_root_clonable: false,
                shadow_root_serializable: false,
                shadow_root_custom_element_registry: None,
                html_for: String::new(),
            },
        );
    Ok(object)
}

pub(crate) fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor");
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<TemplateRecord> {
    scope
        .get_slot::<HtmlTemplateElementStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn update_owner_document(
    scope: &mut v8::PinScope<'_, '_>,
    template: v8::Local<'_, v8::Object>,
    owner_document: v8::Local<'_, v8::Object>,
) {
    let Some(content) = record(scope, template).map(|record| record.content) else {
        return;
    };
    let owner_id = owner_document.get_identity_hash().get();
    let inert_document = if scope
        .get_slot::<HtmlTemplateElementStore>()
        .is_some_and(|store| store.inert_document_ids.contains(&owner_id))
    {
        v8::Global::new(scope, owner_document)
    } else if let Some(existing) = scope
        .get_slot::<HtmlTemplateElementStore>()
        .and_then(|store| store.template_documents.get(&owner_id))
        .cloned()
    {
        existing
    } else {
        let Ok(document) = super::html_document::create(scope) else {
            return;
        };
        let document_id = document.get_identity_hash().get();
        let stored = v8::Global::new(scope, document);
        if let Some(store) = scope.get_slot_mut::<HtmlTemplateElementStore>() {
            store.inert_document_ids.insert(document_id);
            store.template_documents.insert(owner_id, stored.clone());
        }
        stored
    };
    let content = v8::Local::new(scope, &content);
    let inert_document = v8::Local::new(scope, &inert_document);
    super::node::set_owner_document_recursive(scope, content, inert_document);
}

pub(crate) fn get_content(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.content).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn return_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&TemplateRecord) -> &str,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, select(&record)) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn update_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    update: impl FnOnce(&mut TemplateRecord, String),
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    if let Some(record) = scope
        .get_slot_mut::<HtmlTemplateElementStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        update(record, value);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_shadow_root_mode(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |x| &x.shadow_root_mode);
}
pub(crate) fn set_shadow_root_mode(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    update_string(s, a, |x, v| x.shadow_root_mode = v);
}
pub(crate) fn get_html_for(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |x| &x.html_for);
}
pub(crate) fn set_html_for(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    update_string(s, a, |x, v| x.html_for = v);
}

pub(crate) fn return_bool(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&TemplateRecord) -> bool,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, select(&record)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn update_bool(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    update: impl FnOnce(&mut TemplateRecord, bool),
) {
    let value = arguments.get(0).boolean_value(scope);
    if let Some(record) = scope
        .get_slot_mut::<HtmlTemplateElementStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        update(record, value);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_shadow_root_delegates_focus(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_bool(s, a, r, |x| x.shadow_root_delegates_focus);
}
pub(crate) fn set_shadow_root_delegates_focus(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    update_bool(s, a, |x, v| x.shadow_root_delegates_focus = v);
}
pub(crate) fn get_shadow_root_clonable(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_bool(s, a, r, |x| x.shadow_root_clonable);
}
pub(crate) fn set_shadow_root_clonable(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    update_bool(s, a, |x, v| x.shadow_root_clonable = v);
}
pub(crate) fn get_shadow_root_serializable(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_bool(s, a, r, |x| x.shadow_root_serializable);
}
pub(crate) fn set_shadow_root_serializable(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    update_bool(s, a, |x, v| x.shadow_root_serializable = v);
}

pub(crate) fn get_shadow_root_custom_element_registry(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(value) = record.shadow_root_custom_element_registry {
        result.set(v8::Local::new(scope, value).into());
    } else {
        result.set(v8::null(scope).into());
    }
}

pub(crate) fn set_shadow_root_custom_element_registry(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = v8::Local::<v8::Object>::try_from(arguments.get(0)).ok();
    let value = value.map(|value| v8::Global::new(scope, value));
    if let Some(record) = scope
        .get_slot_mut::<HtmlTemplateElementStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.shadow_root_custom_element_registry = value;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
