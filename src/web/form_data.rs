use std::collections::HashMap;

#[derive(Clone)]
struct FormEntry {
    name: String,
    value: v8::Global<v8::Value>,
}

#[derive(Default)]
pub(crate) struct FormDataStore {
    constructors: HashMap<i32, v8::Global<v8::Function>>,
    records: HashMap<i32, Vec<FormEntry>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(FormDataStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "FormData", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<FormDataStore>()
        .and_then(|store| store.constructors.get(&crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "FormData",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_method(scope, prototype, "append", 2, append)?;
    crate::webidl::define_method(scope, prototype, "delete", 1, delete)?;
    crate::webidl::define_method(scope, prototype, "get", 1, get)?;
    crate::webidl::define_method(scope, prototype, "getAll", 1, get_all)?;
    crate::webidl::define_method(scope, prototype, "has", 1, has)?;
    crate::webidl::define_method(scope, prototype, "set", 2, set)?;
    crate::webidl::define_method(scope, prototype, "entries", 0, entries)?;
    crate::webidl::define_method(scope, prototype, "forEach", 1, for_each)?;
    crate::webidl::define_method(scope, prototype, "keys", 0, keys)?;
    crate::webidl::define_method(scope, prototype, "values", 0, values)?;
    let entries_key = crate::webidl::string(scope, "entries")?;
    let entries_function = prototype
        .get(scope, entries_key.into())
        .ok_or_else(|| "cannot read FormData entries".to_owned())?;
    let iterator_symbol = v8::Symbol::get_iterator(scope);
    if prototype.define_own_property(
        scope,
        iterator_symbol.into(),
        entries_function,
        v8::PropertyAttribute::DONT_ENUM,
    ) != Some(true)
    {
        return Err("cannot define FormData iterator".to_owned());
    }
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::move_iterator_to_end(scope, prototype)?;
    let realm_id = crate::webidl::realm_id(scope);
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<FormDataStore>()
        .ok_or_else(|| "FormData state was not prepared".to_owned())?
        .constructors
        .insert(realm_id, stored);
    Ok(constructor)
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'FormData': Please use the 'new' operator",
        );
        return;
    }
    let mut source_form = None;
    let mut submitter = None;
    if !arguments.get(0).is_undefined() && !arguments.get(0).is_null() {
        let Ok(form) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
            crate::webidl::throw_type_error(scope, "parameter 1 is not of type 'HTMLFormElement'");
            return;
        };
        if !super::html_form_element::is_form(scope, form) {
            crate::webidl::throw_type_error(scope, "parameter 1 is not of type 'HTMLFormElement'");
            return;
        }
        if arguments.length() > 1 && !arguments.get(1).is_undefined() {
            let Ok(candidate) = v8::Local::<v8::Object>::try_from(arguments.get(1)) else {
                crate::webidl::throw_type_error(
                    scope,
                    "Failed to construct 'FormData': The submitter is not a submit button.",
                );
                return;
            };
            let is_submit_button = super::html_button_element::record(scope, candidate)
                .is_some_and(|button| button.button_type == "submit")
                || super::html_input_element::record(scope, candidate)
                    .is_some_and(|input| matches!(input.input_type.as_str(), "submit" | "image"));
            if !is_submit_button {
                crate::webidl::throw_type_error(
                    scope,
                    "Failed to construct 'FormData': The submitter is not a submit button.",
                );
                return;
            }
            let owned = super::html_form_element::ancestor_form(scope, candidate)
                .is_some_and(|owner| owner.strict_equals(form.into()));
            if !owned {
                super::node::throw_dom_exception(
                    scope,
                    "NotFoundError",
                    "Failed to construct 'FormData': The specified element is not owned by this form element.",
                );
                return;
            }
            submitter = Some(candidate.get_identity_hash().get());
        }
        source_form = Some(form);
    }
    scope
        .get_slot_mut::<FormDataStore>()
        .expect("FormData state")
        .records
        .insert(arguments.this().get_identity_hash().get(), Vec::new());
    if let Some(form) = source_form {
        populate_from_form(scope, arguments.this(), form, submitter);
    }
    result.set(arguments.this().into());
}

fn populate_from_form(
    scope: &mut v8::PinScope<'_, '_>,
    form_data: v8::Local<'_, v8::Object>,
    form: v8::Local<'_, v8::Object>,
    submitter: Option<i32>,
) {
    for control in super::html_form_element::collect_controls(scope, form) {
        let Some(element) = super::element::record(scope, control) else {
            continue;
        };
        if super::element::attribute_value(scope, control, "disabled").is_some()
            || super::html_element::disabled_by_fieldset(scope, control, &element.tag_name)
        {
            continue;
        }
        match element.tag_name.as_str() {
            "INPUT" => {
                let Some(input) = super::html_input_element::record(scope, control) else {
                    continue;
                };
                if input.name.is_empty()
                    || matches!(input.input_type.as_str(), "button" | "reset")
                    || (matches!(input.input_type.as_str(), "checkbox" | "radio") && !input.checked)
                {
                    continue;
                }
                if input.input_type == "file" {
                    let file_list = v8::Local::new(scope, &input.files);
                    let files = super::file_list::record(scope, file_list).unwrap_or_default();
                    if files.is_empty() {
                        let last_modified = crate::determinism::date_epoch_milliseconds(scope);
                        if let Ok(file) = super::file::create(
                            scope,
                            "",
                            Vec::new(),
                            "application/octet-stream",
                            last_modified,
                        ) {
                            let _ = append_value(scope, form_data, &input.name, file.into());
                        }
                    } else {
                        for file in files {
                            let file = v8::Local::new(scope, &file);
                            let _ = append_value(scope, form_data, &input.name, file.into());
                        }
                    }
                    continue;
                }
                if matches!(input.input_type.as_str(), "image" | "submit")
                    && submitter != Some(control.get_identity_hash().get())
                {
                    continue;
                }
                let value = if matches!(input.input_type.as_str(), "checkbox" | "radio")
                    && !input.value_dirty
                    && super::element::attribute_value(scope, control, "value").is_none()
                {
                    "on".to_owned()
                } else {
                    input.value
                };
                let _ = append_string(scope, form_data, &input.name, &value);
            }
            "SELECT" => {
                let Some(select) = super::html_select_element::record(scope, control) else {
                    continue;
                };
                if select.name.is_empty() {
                    continue;
                }
                super::html_select_element::refresh(scope, control);
                for option in super::html_select_element::options_snapshot(scope, control) {
                    let Some(option_record) = super::html_option_element::record(scope, option)
                    else {
                        continue;
                    };
                    let disabled_by_group =
                        super::node::parent(scope, option).is_some_and(|parent| {
                            super::html_opt_group_element::record(scope, parent)
                                .is_some_and(|record| record.disabled)
                                || super::element::attribute_value(scope, parent, "disabled")
                                    .is_some()
                        });
                    if option_record.selected && !option_record.disabled && !disabled_by_group {
                        if let Some(value) = super::html_option_element::option_value(scope, option)
                        {
                            let _ = append_string(scope, form_data, &select.name, &value);
                        }
                    }
                }
            }
            "TEXTAREA" => {
                let Some(textarea) = super::html_text_area_element::record(scope, control) else {
                    continue;
                };
                let name = textarea.strings.get("name").cloned().unwrap_or_default();
                if name.is_empty() {
                    continue;
                }
                if let Some(value) = super::html_text_area_element::current_value(scope, control) {
                    let _ = append_string(scope, form_data, &name, &value);
                }
            }
            "BUTTON" => {
                let Some(button) = super::html_button_element::record(scope, control) else {
                    continue;
                };
                if submitter == Some(control.get_identity_hash().get())
                    && button.button_type == "submit"
                    && !button.name.is_empty()
                {
                    let _ = append_string(scope, form_data, &button.name, &button.value);
                }
            }
            _ if super::custom_element_registry::is_form_associated(scope, control) => {
                let name =
                    super::element::attribute_value(scope, control, "name").unwrap_or_default();
                if name.is_empty() {
                    continue;
                }
                let Some(value) = super::element_internals::form_value_for_target(scope, control)
                else {
                    continue;
                };
                if super::form_data::is_form_data(
                    scope,
                    v8::Local::<v8::Object>::try_from(value).unwrap_or(control),
                ) {
                    if let Ok(nested) = v8::Local::<v8::Object>::try_from(value) {
                        for entry in snapshot(scope, nested).unwrap_or_default() {
                            let value = v8::Local::new(scope, &entry.value);
                            let _ = append_value(scope, form_data, &entry.name, value);
                        }
                    }
                } else {
                    let _ = append_value(scope, form_data, &name, value);
                }
            }
            _ => {}
        }
    }
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create FormData".to_owned());
    }
    scope
        .get_slot_mut::<FormDataStore>()
        .ok_or_else(|| "FormData state was not prepared".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), Vec::new());
    Ok(object)
}

pub(crate) fn is_form_data(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> bool {
    scope.get_slot::<FormDataStore>().is_some_and(|store| {
        store
            .records
            .contains_key(&object.get_identity_hash().get())
    })
}

pub(crate) fn append_string(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: &str,
) -> bool {
    let Some(value) = v8::String::new(scope, value) else {
        return false;
    };
    let value: v8::Local<'_, v8::Value> = value.into();
    let entry = FormEntry {
        name: name.to_owned(),
        value: v8::Global::new(scope, value),
    };
    let Some(entries) = scope
        .get_slot_mut::<FormDataStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    else {
        return false;
    };
    entries.push(entry);
    true
}

fn append_value(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: v8::Local<'_, v8::Value>,
) -> bool {
    let entry = FormEntry {
        name: name.to_owned(),
        value: v8::Global::new(scope, value),
    };
    let Some(entries) = scope
        .get_slot_mut::<FormDataStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    else {
        return false;
    };
    entries.push(entry);
    true
}

fn snapshot(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<Vec<FormEntry>> {
    scope
        .get_slot::<FormDataStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn entry_value(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> v8::Global<v8::Value> {
    if value.is_object() {
        v8::Global::new(scope, value)
    } else {
        let string = crate::webidl::value_to_string(scope, value);
        let value: v8::Local<'_, v8::Value> = v8::String::new(scope, &string)
            .map(|value| value.into())
            .unwrap_or_else(|| v8::undefined(scope).into());
        v8::Global::new(scope, value)
    }
}

fn required(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: &v8::FunctionCallbackArguments<'_>,
    count: i32,
    method: &str,
) -> bool {
    if arguments.length() < count {
        crate::webidl::throw_type_error(
            scope,
            &format!("Failed to execute '{method}' on 'FormData': {count} arguments required"),
        );
        false
    } else {
        true
    }
}

fn append(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if !required(scope, &arguments, 2, "append") {
        return;
    }
    let name = crate::webidl::value_to_string(scope, arguments.get(0));
    let value = entry_value(scope, arguments.get(1));
    if let Some(entries) = scope.get_slot_mut::<FormDataStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) {
        entries.push(FormEntry { name, value });
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn delete(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if !required(scope, &arguments, 1, "delete") {
        return;
    }
    let name = crate::webidl::value_to_string(scope, arguments.get(0));
    if let Some(entries) = scope.get_slot_mut::<FormDataStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) {
        entries.retain(|entry| entry.name != name);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !required(scope, &arguments, 1, "get") {
        return;
    }
    let name = crate::webidl::value_to_string(scope, arguments.get(0));
    let Some(entries) = snapshot(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(entry) = entries.iter().find(|entry| entry.name == name) {
        result.set(v8::Local::new(scope, &entry.value));
    } else {
        result.set(v8::null(scope).into());
    }
}

fn get_all(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !required(scope, &arguments, 1, "getAll") {
        return;
    }
    let name = crate::webidl::value_to_string(scope, arguments.get(0));
    let Some(entries) = snapshot(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let matched: Vec<FormEntry> = entries
        .into_iter()
        .filter(|entry| entry.name == name)
        .collect();
    let array = v8::Array::new(scope, matched.len() as i32);
    for (index, entry) in matched.iter().enumerate() {
        let value = v8::Local::new(scope, &entry.value);
        let _ = array.set_index(scope, index as u32, value);
    }
    result.set(array.into());
}

fn has(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !required(scope, &arguments, 1, "has") {
        return;
    }
    let name = crate::webidl::value_to_string(scope, arguments.get(0));
    if let Some(entries) = snapshot(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, entries.iter().any(|entry| entry.name == name)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if !required(scope, &arguments, 2, "set") {
        return;
    }
    let name = crate::webidl::value_to_string(scope, arguments.get(0));
    let value = entry_value(scope, arguments.get(1));
    let Some(entries) = scope.get_slot_mut::<FormDataStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(first) = entries.iter().position(|entry| entry.name == name) {
        entries[first].value = value;
        let mut index = entries.len();
        while index > first + 1 {
            index -= 1;
            if entries[index].name == name {
                entries.remove(index);
            }
        }
    } else {
        entries.push(FormEntry { name, value });
    }
}

fn return_iterator(
    scope: &mut v8::PinScope<'_, '_>,
    array: v8::Local<'_, v8::Array>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(key) = v8::String::new(scope, "values") else {
        return;
    };
    let Some(method) = array
        .get(scope, key.into())
        .and_then(|value| v8::Local::<v8::Function>::try_from(value).ok())
    else {
        return;
    };
    if let Some(iterator) = method.call(scope, array.into(), &[]) {
        result.set(iterator);
    }
}

fn entries(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let Some(entries) = snapshot(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let array = v8::Array::new(scope, entries.len() as i32);
    for (index, entry) in entries.iter().enumerate() {
        let pair = v8::Array::new(scope, 2);
        if let Some(name) = v8::String::new(scope, &entry.name) {
            let _ = pair.set_index(scope, 0, name.into());
        }
        let value = v8::Local::new(scope, &entry.value);
        let _ = pair.set_index(scope, 1, value);
        let _ = array.set_index(scope, index as u32, pair.into());
    }
    return_iterator(scope, array, result);
}

fn keys(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let Some(entries) = snapshot(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let array = v8::Array::new(scope, entries.len() as i32);
    for (index, entry) in entries.iter().enumerate() {
        if let Some(name) = v8::String::new(scope, &entry.name) {
            let _ = array.set_index(scope, index as u32, name.into());
        }
    }
    return_iterator(scope, array, result);
}

fn values(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let Some(entries) = snapshot(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let array = v8::Array::new(scope, entries.len() as i32);
    for (index, entry) in entries.iter().enumerate() {
        let value = v8::Local::new(scope, &entry.value);
        let _ = array.set_index(scope, index as u32, value);
    }
    return_iterator(scope, array, result);
}

fn for_each(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if !is_form_data(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    if !required(scope, &arguments, 1, "forEach") {
        return;
    }
    let Ok(callback) = v8::Local::<v8::Function>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "The callback must be a function");
        return;
    };
    let entries = snapshot(scope, arguments.this()).unwrap_or_default();
    let this_value = arguments.get(1);
    for entry in entries {
        let value = v8::Local::new(scope, &entry.value);
        let Some(name) = v8::String::new(scope, &entry.name) else {
            continue;
        };
        let _ = callback.call(
            scope,
            this_value,
            &[value, name.into(), arguments.this().into()],
        );
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<FormDataStore>() {
        store.constructors.remove(&realm_id);
    }
}
