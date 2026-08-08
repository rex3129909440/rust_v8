use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct HtmlOptionElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, OptionRecord>,
}

#[derive(Clone)]
pub(crate) struct OptionRecord {
    pub(crate) text: String,
    pub(crate) value: String,
    pub(crate) label: Option<String>,
    pub(crate) disabled: bool,
    pub(crate) default_selected: bool,
    pub(crate) selected: bool,
    pub(crate) selected_dirty: bool,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(HtmlOptionElementStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "HTMLOptionElement", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<HtmlOptionElementStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "HTMLOptionElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let parent = super::html_element::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;

    super::html_option_element_disabled_property::define(scope, prototype)?;
    super::html_option_element_form_property::define(scope, prototype)?;
    super::html_option_element_label_property::define(scope, prototype)?;
    super::html_option_element_default_selected_property::define(scope, prototype)?;
    super::html_option_element_selected_property::define(scope, prototype)?;
    super::html_option_element_value_property::define(scope, prototype)?;
    super::html_option_element_text_property::define(scope, prototype)?;
    super::html_option_element_index_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;

    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<HtmlOptionElementStore>()
        .ok_or_else(|| "HTMLOptionElement state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, stored);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    text: String,
    value: String,
    default_selected: bool,
    selected: bool,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create HTMLOptionElement".to_owned());
    }
    super::html_element::attach(scope, object, "OPTION");

    let record = OptionRecord {
        text,
        value,
        label: None,
        disabled: false,
        default_selected,
        selected,
        selected_dirty: selected != default_selected,
    };
    scope
        .get_slot_mut::<HtmlOptionElementStore>()
        .expect("HTMLOptionElement state")
        .records
        .insert(object.get_identity_hash().get(), record);
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
) -> Option<OptionRecord> {
    scope
        .get_slot::<HtmlOptionElementStore>()
        .and_then(|store| store.records.get(&object.get_identity_hash().get()))
        .cloned()
}

pub(crate) fn is_option(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> bool {
    record(scope, object).is_some()
}

pub(crate) fn option_value(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<String> {
    record(scope, object)?;
    Some(
        super::element::attribute_value(scope, object, "value")
            .unwrap_or_else(|| super::node::node_text(scope, object)),
    )
}

pub(crate) fn option_selected(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<bool> {
    record(scope, object).map(|record| record.selected)
}

pub(crate) fn reset_selected(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> bool {
    if let Some(record) = scope
        .get_slot_mut::<HtmlOptionElementStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        record.selected = record.default_selected;
        record.selected_dirty = false;
        true
    } else {
        false
    }
}

pub(crate) fn set_option_selected(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    selected: bool,
) -> bool {
    if let Some(record) = scope
        .get_slot_mut::<HtmlOptionElementStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        record.selected = selected;
        record.selected_dirty = true;
        true
    } else {
        false
    }
}

pub(crate) fn attribute_changed(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: Option<&str>,
) {
    let Some(record) = scope
        .get_slot_mut::<HtmlOptionElementStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    else {
        return;
    };
    match name.to_ascii_lowercase().as_str() {
        "disabled" => record.disabled = value.is_some(),
        "label" => record.label = value.map(str::to_owned),
        "selected" => {
            record.default_selected = value.is_some();
            if !record.selected_dirty {
                record.selected = record.default_selected;
            }
        }
        "value" => record.value = value.unwrap_or_default().to_owned(),
        _ => {}
    }
}

pub(crate) fn update(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut OptionRecord),
) {
    if let Some(record) = scope
        .get_slot_mut::<HtmlOptionElementStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        change(record);
    } else {
        crate::webidl::throw_type_error(
            scope,
            "Illegal invocation: receiver is not an HTMLOptionElement",
        );
    }
}

pub(crate) fn get_disabled(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, record.disabled).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_disabled(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0).boolean_value(scope);
    update(scope, arguments.this(), |record| record.disabled = value);
}

pub(crate) fn get_form(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    } else if let Some(form) =
        super::html_select_element::containing_select(scope, arguments.this())
            .and_then(|select| super::html_form_element::ancestor_form(scope, select))
    {
        result.set(form.into());
    } else {
        result.set(v8::null(scope).into());
    }
}

pub(crate) fn get_label(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_some() {
        let value = super::element::attribute_value(scope, arguments.this(), "label")
            .unwrap_or_else(|| super::node::node_text(scope, arguments.this()));
        if let Some(value) = v8::String::new(scope, &value) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_label(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    super::element::set_reflected_string(scope, arguments.this(), "label", value);
}

pub(crate) fn get_default_selected(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(value) = record(scope, arguments.this())
        .and_then(|_| super::element::reflected_boolean(scope, arguments.this(), "selected"))
    {
        result.set(v8::Boolean::new(scope, value).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_default_selected(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let value = arguments.get(0).boolean_value(scope);
    super::element::set_reflected_boolean(scope, arguments.this(), "selected", value);
}

pub(crate) fn get_selected(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, record.selected).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_selected(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0).boolean_value(scope);
    update(scope, arguments.this(), |record| {
        record.selected = value;
        record.selected_dirty = true;
    });
    super::html_select_element::option_selected_by_script(scope, arguments.this(), value);
}

pub(crate) fn get_value(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(value) = option_value(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, &value) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_value(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    super::element::set_reflected_string(scope, arguments.this(), "value", value);
}

pub(crate) fn get_text(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_some() {
        if let Some(value) =
            v8::String::new(scope, &super::node::node_text(scope, arguments.this()))
        {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_text(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    for child in super::node::children(scope, arguments.this()) {
        super::node::detach(scope, child);
    }
    if !value.is_empty()
        && let Ok(text) = super::text::create(scope, value.clone())
    {
        if let Some(document) = super::node::owner_document(scope, arguments.this()) {
            super::node::set_owner_document(scope, text, document);
        }
        let _ = super::node::insert_node(scope, arguments.this(), text, 0);
    }
    update(scope, arguments.this(), |record| record.text = value);
}

pub(crate) fn get_index(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_some() {
        let index = super::html_select_element::containing_select(scope, arguments.this())
            .and_then(|select| {
                super::html_select_element::options_snapshot(scope, select)
                    .iter()
                    .position(|option| option.strict_equals(arguments.this().into()))
            })
            .map(|index| index as i32)
            .unwrap_or(-1);
        result.set(v8::Integer::new(scope, index).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
