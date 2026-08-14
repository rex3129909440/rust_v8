use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum EditHandler {
    TextUpdate,
    TextFormatUpdate,
    CharacterBoundsUpdate,
    CompositionStart,
    CompositionEnd,
}

#[derive(Clone)]
struct EditContextRecord {
    text: String,
    selection_start: u32,
    selection_end: u32,
    character_bounds_range_start: u32,
    character_bounds: Vec<v8::Global<v8::Value>>,
    attached_elements: Vec<v8::Global<v8::Object>>,
    handlers: HashMap<EditHandler, v8::Global<v8::Function>>,
}

#[derive(Default)]
pub(crate) struct EditContextStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, EditContextRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(EditContextStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "EditContext", c.into())
}
fn ensure_constructor<'s>(
    s: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(c) = s
        .get_slot::<EditContextStore>()
        .and_then(|x| x.constructor.get(crate::webidl::realm_id(s)))
        .cloned()
    {
        return Ok(v8::Local::new(s, &c));
    }
    let c = crate::webidl::create_function(
        s,
        "EditContext",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let parent = super::event_target::ensure_constructor(s)?;
    crate::webidl::inherit(s, c, parent)?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_readonly_accessor(s, p, "text", get_text)?;
    crate::webidl::define_readonly_accessor(s, p, "selectionStart", get_selection_start)?;
    crate::webidl::define_readonly_accessor(s, p, "selectionEnd", get_selection_end)?;
    crate::webidl::define_readonly_accessor(
        s,
        p,
        "characterBoundsRangeStart",
        get_character_bounds_range_start,
    )?;
    crate::webidl::define_accessor(s, p, "ontextupdate", get_on_text_update, set_on_text_update)?;
    crate::webidl::define_accessor(
        s,
        p,
        "ontextformatupdate",
        get_on_text_format_update,
        set_on_text_format_update,
    )?;
    crate::webidl::define_accessor(
        s,
        p,
        "oncharacterboundsupdate",
        get_on_character_bounds_update,
        set_on_character_bounds_update,
    )?;
    crate::webidl::define_accessor(
        s,
        p,
        "oncompositionstart",
        get_on_composition_start,
        set_on_composition_start,
    )?;
    crate::webidl::define_accessor(
        s,
        p,
        "oncompositionend",
        get_on_composition_end,
        set_on_composition_end,
    )?;
    crate::webidl::define_method(s, p, "attachedElements", 0, attached_elements)?;
    crate::webidl::define_method(s, p, "characterBounds", 0, character_bounds)?;
    crate::webidl::define_method(s, p, "updateCharacterBounds", 2, update_character_bounds)?;
    crate::webidl::define_method(s, p, "updateControlBounds", 1, update_control_bounds)?;
    crate::webidl::define_method(s, p, "updateSelection", 2, update_selection)?;
    crate::webidl::define_method(s, p, "updateSelectionBounds", 1, update_selection_bounds)?;
    crate::webidl::define_method(s, p, "updateText", 3, update_text)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let realm_id = crate::webidl::realm_id(s);
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<EditContextStore>()
        .ok_or_else(|| "EditContext state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
fn construct(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !a.is_construct_call() {
        crate::webidl::throw_type_error(s, "Please use the 'new' operator");
        return;
    }
    let init = v8::Local::<v8::Object>::try_from(a.get(0)).ok();
    let text = init
        .and_then(|o| property(s, o, "text"))
        .filter(|v| !v.is_undefined())
        .map(|v| crate::webidl::value_to_string(s, v))
        .unwrap_or_default();
    let max = text.encode_utf16().count() as u32;
    let start = init
        .and_then(|o| number_property(s, o, "selectionStart"))
        .unwrap_or(0)
        .min(max);
    let end = init
        .and_then(|o| number_property(s, o, "selectionEnd"))
        .unwrap_or(start)
        .min(max);
    super::event_target::attach(s, a.this());
    s.get_slot_mut::<EditContextStore>()
        .expect("EditContext state")
        .records
        .insert(
            a.this().get_identity_hash().get(),
            EditContextRecord {
                text,
                selection_start: start,
                selection_end: end,
                character_bounds_range_start: 0,
                character_bounds: Vec::new(),
                attached_elements: Vec::new(),
                handlers: HashMap::new(),
            },
        );
    r.set(a.this().into())
}
fn property<'s>(
    s: &v8::PinScope<'s, '_>,
    o: v8::Local<'s, v8::Object>,
    name: &str,
) -> Option<v8::Local<'s, v8::Value>> {
    let key = v8::String::new(s, name)?;
    o.get(s, key.into())
}
fn number_property(
    s: &v8::PinScope<'_, '_>,
    o: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<u32> {
    property(s, o, name)?.uint32_value(s)
}
fn record(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<EditContextRecord> {
    s.get_slot::<EditContextStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
fn get_text(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(x) = record(s, a.this()) {
        if let Some(v) = v8::String::new(s, &x.text) {
            r.set(v.into())
        }
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn number_get(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
    select: impl FnOnce(&EditContextRecord) -> u32,
) {
    if let Some(x) = record(s, a.this()) {
        r.set(v8::Integer::new_from_unsigned(s, select(&x)).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_selection_start(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    number_get(s, a, r, |x| x.selection_start)
}
fn get_selection_end(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    number_get(s, a, r, |x| x.selection_end)
}
fn get_character_bounds_range_start(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    number_get(s, a, r, |x| x.character_bounds_range_start)
}
fn get_handler(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
    slot: EditHandler,
) {
    match record(s, a.this()) {
        Some(x) => match x.handlers.get(&slot) {
            Some(v) => r.set(v8::Local::new(s, v).into()),
            None => r.set(v8::null(s).into()),
        },
        None => crate::webidl::throw_type_error(s, "Illegal invocation"),
    }
}
fn set_handler(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    slot: EditHandler,
) {
    let value = v8::Local::<v8::Function>::try_from(a.get(0))
        .ok()
        .map(|v| v8::Global::new(s, v));
    let Some(x) = s
        .get_slot_mut::<EditContextStore>()
        .and_then(|q| q.records.get_mut(&a.this().get_identity_hash().get()))
    else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    if let Some(value) = value {
        x.handlers.insert(slot, value);
    } else {
        x.handlers.remove(&slot);
    }
}
fn get_on_text_update(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_handler(s, a, r, EditHandler::TextUpdate)
}
fn set_on_text_update(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, EditHandler::TextUpdate)
}
fn get_on_text_format_update(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_handler(s, a, r, EditHandler::TextFormatUpdate)
}
fn set_on_text_format_update(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, EditHandler::TextFormatUpdate)
}
fn get_on_character_bounds_update(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_handler(s, a, r, EditHandler::CharacterBoundsUpdate)
}
fn set_on_character_bounds_update(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, EditHandler::CharacterBoundsUpdate)
}
fn get_on_composition_start(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_handler(s, a, r, EditHandler::CompositionStart)
}
fn set_on_composition_start(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, EditHandler::CompositionStart)
}
fn get_on_composition_end(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_handler(s, a, r, EditHandler::CompositionEnd)
}
fn set_on_composition_end(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, EditHandler::CompositionEnd)
}
fn attached_elements(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(x) = record(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    let out = v8::Array::new(s, x.attached_elements.len() as i32);
    for (index, value) in x.attached_elements.iter().enumerate() {
        let _ = out.set_index(s, index as u32, v8::Local::new(s, value).into());
    }
    r.set(out.into())
}
fn character_bounds(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(x) = record(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    let out = v8::Array::new(s, x.character_bounds.len() as i32);
    for (index, value) in x.character_bounds.iter().enumerate() {
        let _ = out.set_index(s, index as u32, v8::Local::new(s, value));
    }
    r.set(out.into())
}
fn update_character_bounds(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_none() {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    }
    if a.length() < 2 {
        crate::webidl::throw_type_error(s, "2 arguments required");
        return;
    }
    let start = a.get(0).uint32_value(s).unwrap_or(0);
    let Ok(values) = v8::Local::<v8::Object>::try_from(a.get(1)) else {
        crate::webidl::throw_type_error(s, "bounds must be a sequence");
        return;
    };
    let length = property(s, values, "length")
        .and_then(|v| v.uint32_value(s))
        .unwrap_or(0);
    let mut bounds = Vec::new();
    for index in 0..length {
        if let Some(value) = values.get_index(s, index) {
            bounds.push(v8::Global::new(s, value));
        }
    }
    if let Some(x) = s
        .get_slot_mut::<EditContextStore>()
        .and_then(|q| q.records.get_mut(&a.this().get_identity_hash().get()))
    {
        x.character_bounds_range_start = start;
        x.character_bounds = bounds
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn update_selection(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let start = a.get(0).uint32_value(s).unwrap_or(0);
    let end = a.get(1).uint32_value(s).unwrap_or(start);
    if let Some(x) = s
        .get_slot_mut::<EditContextStore>()
        .and_then(|q| q.records.get_mut(&a.this().get_identity_hash().get()))
    {
        let max = x.text.encode_utf16().count() as u32;
        x.selection_start = start.min(max);
        x.selection_end = end.min(max)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn update_text(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if a.length() < 3 {
        crate::webidl::throw_type_error(s, "3 arguments required");
        return;
    }
    let start = a.get(0).uint32_value(s).unwrap_or(0) as usize;
    let end = a.get(1).uint32_value(s).unwrap_or(start as u32) as usize;
    let replacement = crate::webidl::value_to_string(s, a.get(2));
    if let Some(x) = s
        .get_slot_mut::<EditContextStore>()
        .and_then(|q| q.records.get_mut(&a.this().get_identity_hash().get()))
    {
        let mut units = x.text.encode_utf16().collect::<Vec<_>>();
        let start = start.min(units.len());
        let end = end.min(units.len()).max(start);
        units.splice(start..end, replacement.encode_utf16());
        x.text = String::from_utf16_lossy(&units);
        let caret = (start + replacement.encode_utf16().count()) as u32;
        x.selection_start = caret;
        x.selection_end = caret
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn update_control_bounds(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_none() {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    } else if a.length() < 1 {
        crate::webidl::throw_type_error(s, "1 argument required")
    }
}
fn update_selection_bounds(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_none() {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    } else if a.length() < 1 {
        crate::webidl::throw_type_error(s, "1 argument required")
    }
}
