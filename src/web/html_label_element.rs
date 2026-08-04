use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct HtmlLabelElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) html_for: HashMap<i32, String>,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(HtmlLabelElementStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "HTMLLabelElement", constructor.into())
}
pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(constructor) = scope
        .get_slot::<HtmlLabelElementStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let parent = super::html_element::ensure_constructor(scope)?;
    let constructor = crate::webidl::create_function(
        scope,
        "HTMLLabelElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::html_label_element_form_property::define(scope, prototype)?;
    super::html_label_element_html_for_property::define(scope, prototype)?;
    super::html_label_element_control_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<HtmlLabelElementStore>()
        .ok_or_else(|| "HTMLLabelElement state was not prepared".to_owned())?
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
        return Err("cannot create HTMLLabelElement".to_owned());
    }
    super::html_element::attach(scope, object, "LABEL");
    scope
        .get_slot_mut::<HtmlLabelElementStore>()
        .ok_or_else(|| "HTMLLabelElement state was not prepared".to_owned())?
        .html_for
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
pub(crate) fn value(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<String> {
    scope
        .get_slot::<HtmlLabelElementStore>()?
        .html_for
        .contains_key(&object.get_identity_hash().get())
        .then(|| super::element::attribute_value(scope, object, "for").unwrap_or_default())
}
pub(crate) fn get_form(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if value(scope, a.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    } else if let Some(form) = super::html_form_element::ancestor_form(scope, a.this()) {
        r.set(form.into());
    } else {
        r.set(v8::null(scope).into());
    }
}
pub(crate) fn get_html_for(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(current) = value(scope, a.this()) {
        if let Some(current) = v8::String::new(scope, &current) {
            r.set(current.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn set_html_for(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let next = crate::webidl::value_to_string(scope, a.get(0));
    if value(scope, a.this()).is_some() {
        super::element::set_reflected_string(scope, a.this(), "for", next);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn is_labellable(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> bool {
    super::element::record(scope, object).is_some_and(|record| match record.tag_name.as_str() {
        "BUTTON" | "METER" | "OUTPUT" | "PROGRESS" | "SELECT" | "TEXTAREA" => true,
        "INPUT" => !super::element::attribute_value(scope, object, "type")
            .is_some_and(|value| value.eq_ignore_ascii_case("hidden")),
        _ => false,
    })
}
pub(crate) fn find_by_id<'s>(
    scope: &v8::PinScope<'s, '_>,
    root: v8::Local<'s, v8::Object>,
    id: &str,
) -> Option<v8::Local<'s, v8::Object>> {
    if super::element::record(scope, root).is_some_and(|record| {
        record
            .attributes
            .iter()
            .any(|(name, value)| name.eq_ignore_ascii_case("id") && value == id)
    }) {
        return Some(root);
    }
    for child in super::node::children(scope, root) {
        if let Some(found) = find_by_id(scope, child, id) {
            return Some(found);
        }
    }
    None
}
pub(crate) fn find_descendant_control<'s>(
    scope: &v8::PinScope<'s, '_>,
    root: v8::Local<'s, v8::Object>,
) -> Option<v8::Local<'s, v8::Object>> {
    for child in super::node::children(scope, root) {
        if is_labellable(scope, child) {
            return Some(child);
        }
        if let Some(found) = find_descendant_control(scope, child) {
            return Some(found);
        }
    }
    None
}

pub(crate) fn control_for_label<'s>(
    scope: &v8::PinScope<'s, '_>,
    label: v8::Local<'s, v8::Object>,
) -> Option<v8::Local<'s, v8::Object>> {
    let html_for = value(scope, label)?;
    if html_for.is_empty() {
        return find_descendant_control(scope, label);
    }
    if super::node::is_connected(scope, label)
        && let Some(document) = super::node::owner_document(scope, label)
    {
        return super::document::document_descendants(scope, document)
            .into_iter()
            .find(|candidate| {
                super::element::attribute_value(scope, *candidate, "id").as_deref()
                    == Some(html_for.as_str())
            })
            .filter(|candidate| is_labellable(scope, *candidate));
    }
    let mut root = label;
    while let Some(parent) = super::node::parent(scope, root) {
        root = parent;
    }
    find_by_id(scope, root, &html_for).filter(|candidate| is_labellable(scope, *candidate))
}

pub(crate) fn labels_for<'s>(
    scope: &v8::PinScope<'s, '_>,
    control: v8::Local<'s, v8::Object>,
) -> Vec<v8::Local<'s, v8::Object>> {
    if !is_labellable(scope, control) {
        return Vec::new();
    }
    let candidates = if super::node::is_connected(scope, control) {
        super::node::owner_document(scope, control)
            .map(|document| super::document::document_descendants(scope, document))
            .unwrap_or_default()
    } else {
        let mut root = control;
        while let Some(parent) = super::node::parent(scope, root) {
            root = parent;
        }
        let mut descendants = super::dom_selector::descendants(scope, root);
        descendants.insert(0, root);
        descendants
    };
    let control_id = control.get_identity_hash().get();
    candidates
        .into_iter()
        .filter(|candidate| {
            value(scope, *candidate).is_some()
                && control_for_label(scope, *candidate)
                    .is_some_and(|found| found.get_identity_hash().get() == control_id)
        })
        .collect()
}

pub(crate) fn get_control(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if value(scope, a.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let control = control_for_label(scope, a.this());
    if let Some(control) = control {
        r.set(control.into());
    } else {
        r.set(v8::null(scope).into());
    }
}
