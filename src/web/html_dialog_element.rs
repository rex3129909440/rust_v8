use std::collections::HashMap;

#[derive(Clone)]
pub(crate) struct DialogRecord {
    pub(crate) open: bool,
    pub(crate) return_value: String,
    pub(crate) closed_by: String,
    pub(crate) modal: bool,
}

impl Default for DialogRecord {
    fn default() -> Self {
        Self {
            open: false,
            return_value: String::new(),
            closed_by: "none".to_owned(),
            modal: false,
        }
    }
}

#[derive(Default)]
pub(crate) struct HtmlDialogElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, DialogRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(HtmlDialogElementStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "HTMLDialogElement", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(constructor) = scope
        .get_slot::<HtmlDialogElementStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let parent = super::html_element::ensure_constructor(scope)?;
    let constructor = crate::webidl::create_function(
        scope,
        "HTMLDialogElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::html_dialog_element_open_property::define(scope, prototype)?;
    super::html_dialog_element_return_value_property::define(scope, prototype)?;
    super::html_dialog_element_closed_by_property::define(scope, prototype)?;
    super::html_dialog_element_close::define(scope, prototype)?;
    super::html_dialog_element_request_close::define(scope, prototype)?;
    super::html_dialog_element_show::define(scope, prototype)?;
    super::html_dialog_element_show_modal::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<HtmlDialogElementStore>()
        .ok_or_else(|| "HTMLDialogElement state was not prepared".to_owned())?
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
        return Err("cannot create HTMLDialogElement".to_owned());
    }
    super::html_element::attach(scope, object, "DIALOG");
    scope
        .get_slot_mut::<HtmlDialogElementStore>()
        .ok_or_else(|| "HTMLDialogElement state was not prepared".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), DialogRecord::default());
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
) -> Option<DialogRecord> {
    scope
        .get_slot::<HtmlDialogElementStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn update(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut DialogRecord),
) {
    if let Some(record) = scope
        .get_slot_mut::<HtmlDialogElementStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        change(record);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_open(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, record.open).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_open(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0).boolean_value(scope);
    update(scope, arguments.this(), |record| {
        record.open = value;
        if !value {
            record.modal = false;
        }
    });
}

pub(crate) fn get_return_value(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, &record.return_value) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_return_value(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    update(scope, arguments.this(), |record| {
        record.return_value = value
    });
}

pub(crate) fn get_closed_by(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, &record.closed_by) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_closed_by(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0)).to_ascii_lowercase();
    let normalized = match value.as_str() {
        "any" => "any",
        "closerequest" => "closerequest",
        _ => "none",
    }
    .to_owned();
    update(scope, arguments.this(), |record| {
        record.closed_by = normalized
    });
}

pub(crate) fn close(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(current) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if !current.open {
        return;
    }
    let return_value = if arguments.length() > 0 {
        Some(crate::webidl::value_to_string(scope, arguments.get(0)))
    } else {
        None
    };
    update(scope, arguments.this(), |record| {
        record.open = false;
        record.modal = false;
        if let Some(value) = return_value {
            record.return_value = value;
        }
    });
    let event = super::event_target::create_event(scope, "close");
    super::event_target::dispatch(scope, arguments.this(), event);
}

pub(crate) fn request_close(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    if !record(scope, arguments.this()).is_some_and(|record| record.open) {
        return;
    }
    let cancel_event = super::event_target::create_event(scope, "cancel");
    if super::event_target::dispatch(scope, arguments.this(), cancel_event) {
        close(scope, arguments, result);
    }
}

pub(crate) fn show(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(current) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if current.open {
        if current.modal {
            crate::webidl::throw_type_error(scope, "The dialog is already open as a modal dialog");
        }
        return;
    }
    update(scope, arguments.this(), |record| {
        record.open = true;
        record.modal = false;
    });
}

pub(crate) fn show_modal(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(current) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if current.open {
        if !current.modal {
            crate::webidl::throw_type_error(
                scope,
                "The dialog is already open as a non-modal dialog",
            );
        }
        return;
    }
    update(scope, arguments.this(), |record| {
        record.open = true;
        record.modal = true;
    });
}
