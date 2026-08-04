use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct InputEventStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, InputEventRecord>,
}

#[derive(Clone)]
pub(crate) struct InputEventRecord {
    pub(crate) data: Option<String>,
    pub(crate) is_composing: bool,
    pub(crate) input_type: String,
    pub(crate) data_transfer: Option<v8::Global<v8::Value>>,
    pub(crate) target_ranges: Vec<v8::Global<v8::Value>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(InputEventStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "InputEvent", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<InputEventStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "InputEvent",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::input_event_data_property::define(scope, prototype)?;
    super::input_event_is_composing_property::define(scope, prototype)?;
    super::input_event_input_type_property::define(scope, prototype)?;
    super::input_event_data_transfer_property::define(scope, prototype)?;
    super::input_event_get_target_ranges::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::ui_event::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<InputEventStore>()
        .ok_or_else(|| "InputEvent state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'InputEvent': 1 argument required",
        );
        return;
    }
    let init = v8::Local::<v8::Object>::try_from(arguments.get(1)).ok();
    let bubbles = init.is_some_and(|value| super::event::boolean_property(scope, value, "bubbles"));
    let cancelable =
        init.is_some_and(|value| super::event::boolean_property(scope, value, "cancelable"));
    let composed =
        init.is_some_and(|value| super::event::boolean_property(scope, value, "composed"));
    let view = init
        .and_then(|value| property(scope, value, "view"))
        .filter(|value| !value.is_null_or_undefined())
        .map(|value| v8::Global::new(scope, value));
    let detail = init
        .map(|value| super::event::number_property(scope, value, "detail", 0.0) as i32)
        .unwrap_or(0);
    let source_capabilities = init
        .and_then(|value| property(scope, value, "sourceCapabilities"))
        .filter(|value| !value.is_null_or_undefined())
        .map(|value| v8::Global::new(scope, value));
    super::ui_event::attach(
        scope,
        arguments.this(),
        crate::webidl::value_to_string(scope, arguments.get(0)),
        bubbles,
        cancelable,
        composed,
        view,
        detail,
        source_capabilities,
    );
    let data_value = init.and_then(|value| property(scope, value, "data"));
    let data = data_value
        .filter(|value| !value.is_null_or_undefined())
        .map(|value| crate::webidl::value_to_string(scope, value));
    let is_composing =
        init.is_some_and(|value| super::event::boolean_property(scope, value, "isComposing"));
    let input_type = init
        .and_then(|value| property(scope, value, "inputType"))
        .filter(|value| !value.is_undefined())
        .map(|value| sanitize_input_type(&crate::webidl::value_to_string(scope, value)))
        .unwrap_or_default();
    let data_transfer = init
        .and_then(|value| property(scope, value, "dataTransfer"))
        .filter(|value| !value.is_null_or_undefined())
        .map(|value| v8::Global::new(scope, value));
    let target_ranges = init
        .and_then(|value| property(scope, value, "targetRanges"))
        .map(|value| read_sequence(scope, value))
        .unwrap_or_default();
    scope
        .get_slot_mut::<InputEventStore>()
        .expect("InputEvent state")
        .records
        .insert(
            arguments.this().get_identity_hash().get(),
            InputEventRecord {
                data,
                is_composing,
                input_type,
                data_transfer,
                target_ranges,
            },
        );
    result.set(arguments.this().into());
}

pub(crate) fn sanitize_input_type(value: &str) -> String {
    match value {
        ""
        | "insertText"
        | "insertReplacementText"
        | "insertLineBreak"
        | "insertParagraph"
        | "insertOrderedList"
        | "insertUnorderedList"
        | "insertHorizontalRule"
        | "insertFromYank"
        | "insertFromDrop"
        | "insertFromPaste"
        | "insertFromPasteAsQuotation"
        | "insertTranspose"
        | "insertCompositionText"
        | "insertLink"
        | "deleteWordBackward"
        | "deleteWordForward"
        | "deleteSoftLineBackward"
        | "deleteSoftLineForward"
        | "deleteEntireSoftLine"
        | "deleteHardLineBackward"
        | "deleteHardLineForward"
        | "deleteByDrag"
        | "deleteByCut"
        | "deleteContent"
        | "deleteContentBackward"
        | "deleteContentForward"
        | "historyUndo"
        | "historyRedo"
        | "formatBold"
        | "formatItalic"
        | "formatUnderline"
        | "formatStrikeThrough"
        | "formatSuperscript"
        | "formatSubscript"
        | "formatJustifyFull"
        | "formatJustifyCenter"
        | "formatJustifyRight"
        | "formatJustifyLeft"
        | "formatIndent"
        | "formatOutdent"
        | "formatRemove"
        | "formatSetBlockTextDirection"
        | "formatSetInlineTextDirection"
        | "formatBackColor"
        | "formatFontColor"
        | "formatFontName" => value.to_owned(),
        _ => String::new(),
    }
}

pub(crate) fn property<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'s, v8::Object>,
    name: &str,
) -> Option<v8::Local<'s, v8::Value>> {
    object.get(scope, v8::String::new(scope, name)?.into())
}

pub(crate) fn read_sequence(
    scope: &v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> Vec<v8::Global<v8::Value>> {
    let Ok(object) = v8::Local::<v8::Object>::try_from(value) else {
        return Vec::new();
    };
    let Some(length) =
        property(scope, object, "length").and_then(|value| value.uint32_value(scope))
    else {
        return Vec::new();
    };
    let mut values = Vec::with_capacity(length as usize);
    for index in 0..length {
        if let Some(value) = object.get_index(scope, index) {
            values.push(v8::Global::new(scope, value));
        }
    }
    values
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<InputEventRecord> {
    scope
        .get_slot::<InputEventStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn get_data(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match record(scope, arguments.this()) {
        Some(record) => match record.data {
            Some(data) => {
                if let Some(data) = v8::String::new(scope, &data) {
                    result.set(data.into());
                }
            }
            None => result.set(v8::null(scope).into()),
        },
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}

pub(crate) fn get_is_composing(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, record.is_composing).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_input_type(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, &record.input_type) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_data_transfer(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match record(scope, arguments.this()) {
        Some(record) => match record.data_transfer {
            Some(value) => result.set(v8::Local::new(scope, &value)),
            None => result.set(v8::null(scope).into()),
        },
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}

pub(crate) fn get_target_ranges(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let output = v8::Array::new(scope, record.target_ranges.len() as i32);
    for (index, value) in record.target_ranges.iter().enumerate() {
        let _ = output.set_index(scope, index as u32, v8::Local::new(scope, value));
    }
    result.set(output.into());
}
