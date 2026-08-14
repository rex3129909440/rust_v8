use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct VttCueStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, VttCueRecord>,
}

#[derive(Clone)]
enum LineValue {
    Auto,
    Number(f64),
}

#[derive(Clone)]
struct VttCueRecord {
    vertical: String,
    snap_to_lines: bool,
    line: LineValue,
    position: f64,
    size: f64,
    align: String,
    text: String,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(VttCueStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "VTTCue", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<VttCueStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "VTTCue",
        3,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_accessor(scope, prototype, "vertical", get_vertical, set_vertical)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "snapToLines",
        get_snap_to_lines,
        set_snap_to_lines,
    )?;
    crate::webidl::define_accessor(scope, prototype, "line", get_line, set_line)?;
    crate::webidl::define_accessor(scope, prototype, "position", get_position, set_position)?;
    crate::webidl::define_accessor(scope, prototype, "size", get_size, set_size)?;
    crate::webidl::define_accessor(scope, prototype, "align", get_align, set_align)?;
    crate::webidl::define_accessor(scope, prototype, "text", get_text, set_text)?;
    crate::webidl::define_method(scope, prototype, "getCueAsHTML", 0, get_cue_as_html)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let text_track_cue = super::text_track_cue::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, text_track_cue)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<VttCueStore>()
        .ok_or_else(|| "VTTCue state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 3 {
        crate::webidl::throw_type_error(scope, "VTTCue requires startTime, endTime, and text");
        return;
    }
    if let Some(message) = crate::webidl::number_conversion_error(arguments.get(0)) {
        crate::webidl::throw_type_error(scope, &message);
        return;
    }
    let Some(start_time) = arguments.get(0).number_value(scope) else {
        return;
    };
    if let Some(message) = crate::webidl::number_conversion_error(arguments.get(1)) {
        crate::webidl::throw_type_error(scope, &message);
        return;
    }
    let Some(end_time) = arguments.get(1).number_value(scope) else {
        return;
    };
    if !start_time.is_finite() || !end_time.is_finite() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'VTTCue': The provided double value is non-finite.",
        );
        return;
    }
    let text = crate::webidl::value_to_string(scope, arguments.get(2));
    let object = arguments.this();
    super::text_track_cue::attach(scope, object, start_time, end_time);
    if let Some(store) = scope.get_slot_mut::<VttCueStore>() {
        store.records.insert(
            object.get_identity_hash().get(),
            VttCueRecord {
                vertical: String::new(),
                snap_to_lines: true,
                line: LineValue::Auto,
                position: 50.0,
                size: 100.0,
                align: "center".to_owned(),
                text,
            },
        );
    }
    result.set(object.into());
}

fn record(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> Option<VttCueRecord> {
    scope
        .get_slot::<VttCueStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}
fn update(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut VttCueRecord),
) {
    if let Some(record) = scope
        .get_slot_mut::<VttCueStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        change(record)
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}

fn return_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&VttCueRecord) -> &str,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, select(&record)) {
            result.set(value.into())
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn get_vertical(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |v| &v.vertical)
}
fn get_align(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |v| &v.align)
}
fn get_text(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |v| &v.text)
}

fn set_vertical(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_none() {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    }
    let value = crate::webidl::value_to_string(s, a.get(0));
    if value != "" && value != "rl" && value != "lr" {
        crate::webidl::throw_type_error(s, "Invalid VTTCue vertical value");
        return;
    }
    update(s, a.this(), |v| v.vertical = value)
}
fn set_align(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_none() {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    }
    let value = crate::webidl::value_to_string(s, a.get(0));
    if !["start", "center", "end", "left", "right"].contains(&value.as_str()) {
        crate::webidl::throw_type_error(s, "Invalid VTTCue align value");
        return;
    }
    update(s, a.this(), |v| v.align = value)
}
fn set_text(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(s, a.get(0));
    update(s, a.this(), |v| v.text = value)
}

fn get_snap_to_lines(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Boolean::new(s, v.snap_to_lines).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn set_snap_to_lines(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = a.get(0).boolean_value(s);
    update(s, a.this(), |v| v.snap_to_lines = value)
}

fn get_line(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        match v.line {
            LineValue::Auto => {
                if let Some(value) = v8::String::new(s, "auto") {
                    r.set(value.into())
                }
            }
            LineValue::Number(value) => r.set(v8::Number::new(s, value).into()),
        }
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn set_line(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = if a.get(0).is_string() && crate::webidl::value_to_string(s, a.get(0)) == "auto" {
        LineValue::Auto
    } else {
        LineValue::Number(a.get(0).number_value(s).unwrap_or(f64::NAN))
    };
    update(s, a.this(), |v| v.line = value)
}

fn return_number(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&VttCueRecord) -> f64,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, select(&record)).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn get_position(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.position)
}
fn get_size(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.size)
}
fn set_position(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = a.get(0).number_value(s).unwrap_or(f64::NAN);
    if !(0.0..=100.0).contains(&value) {
        crate::webidl::throw_type_error(s, "VTTCue position must be between 0 and 100");
        return;
    }
    update(s, a.this(), |v| v.position = value)
}
fn set_size(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = a.get(0).number_value(s).unwrap_or(f64::NAN);
    if !(0.0..=100.0).contains(&value) {
        crate::webidl::throw_type_error(s, "VTTCue size must be between 0 and 100");
        return;
    }
    update(s, a.this(), |v| v.size = value)
}

fn get_cue_as_html(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(record) = record(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    let fragment = v8::Object::new(s);
    if let (Some(key), Some(value)) = (
        v8::String::new(s, "textContent"),
        v8::String::new(s, &record.text),
    ) {
        let _ = fragment.create_data_property(s, key.into(), value.into());
    }
    if let Some(tag) = v8::String::new(s, "DocumentFragment") {
        let symbol = v8::Symbol::get_to_string_tag(s);
        let _ = fragment.define_own_property(
            s,
            symbol.into(),
            tag.into(),
            v8::PropertyAttribute::READ_ONLY | v8::PropertyAttribute::DONT_ENUM,
        );
    }
    r.set(fragment.into())
}
