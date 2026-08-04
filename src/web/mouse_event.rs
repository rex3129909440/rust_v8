use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct MouseEventStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, MouseEventData>,
}

#[derive(Clone)]
pub(crate) struct MouseEventData {
    pub(crate) event_type: String,
    pub(crate) screen_x: i32,
    pub(crate) screen_y: i32,
    pub(crate) client_x: i32,
    pub(crate) client_y: i32,
    pub(crate) ctrl_key: bool,
    pub(crate) shift_key: bool,
    pub(crate) alt_key: bool,
    pub(crate) meta_key: bool,
    pub(crate) button: i16,
    pub(crate) buttons: u16,
    pub(crate) related_target: Option<v8::Global<v8::Value>>,
    pub(crate) movement_x: i32,
    pub(crate) movement_y: i32,
    pub(crate) bubbles: bool,
    pub(crate) cancelable: bool,
    pub(crate) composed: bool,
    pub(crate) view: Option<v8::Global<v8::Value>>,
    pub(crate) detail: i32,
}

impl Default for MouseEventData {
    fn default() -> Self {
        Self {
            event_type: String::new(),
            screen_x: 0,
            screen_y: 0,
            client_x: 0,
            client_y: 0,
            ctrl_key: false,
            shift_key: false,
            alt_key: false,
            meta_key: false,
            button: 0,
            buttons: 0,
            related_target: None,
            movement_x: 0,
            movement_y: 0,
            bubbles: false,
            cancelable: false,
            composed: false,
            view: None,
            detail: 0,
        }
    }
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(MouseEventStore::default());
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<MouseEventStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let parent = super::ui_event::ensure_constructor(scope)?;
    let constructor = crate::webidl::create_function(
        scope,
        "MouseEvent",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::mouse_event_screen_x_property::define(scope, prototype)?;
    super::mouse_event_screen_y_property::define(scope, prototype)?;
    super::mouse_event_client_x_property::define(scope, prototype)?;
    super::mouse_event_client_y_property::define(scope, prototype)?;
    super::mouse_event_ctrl_key_property::define(scope, prototype)?;
    super::mouse_event_shift_key_property::define(scope, prototype)?;
    super::mouse_event_alt_key_property::define(scope, prototype)?;
    super::mouse_event_meta_key_property::define(scope, prototype)?;
    super::mouse_event_button_property::define(scope, prototype)?;
    super::mouse_event_buttons_property::define(scope, prototype)?;
    super::mouse_event_related_target_property::define(scope, prototype)?;
    super::mouse_event_page_x_property::define(scope, prototype)?;
    super::mouse_event_page_y_property::define(scope, prototype)?;
    super::mouse_event_x_property::define(scope, prototype)?;
    super::mouse_event_y_property::define(scope, prototype)?;
    super::mouse_event_offset_x_property::define(scope, prototype)?;
    super::mouse_event_offset_y_property::define(scope, prototype)?;
    super::mouse_event_movement_x_property::define(scope, prototype)?;
    super::mouse_event_movement_y_property::define(scope, prototype)?;
    super::mouse_event_from_element_property::define(scope, prototype)?;
    super::mouse_event_to_element_property::define(scope, prototype)?;
    super::mouse_event_layer_x_property::define(scope, prototype)?;
    super::mouse_event_layer_y_property::define(scope, prototype)?;
    super::mouse_event_get_modifier_state::define(scope, prototype)?;
    super::mouse_event_init_mouse_event::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<MouseEventStore>()
        .ok_or_else(|| "MouseEvent state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

#[allow(dead_code)]
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "MouseEvent", constructor.into())
}

pub(crate) fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'MouseEvent': Please use the 'new' operator",
        );
        return;
    }
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'MouseEvent': 1 argument required",
        );
        return;
    }
    let event_type = crate::webidl::value_to_string(scope, arguments.get(0));
    let data = read_init(scope, arguments.get(1));
    attach(scope, arguments.this(), event_type, data);
    result.set(arguments.this().into());
}

pub(crate) fn read_init(
    scope: &v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> MouseEventData {
    let Ok(object) = v8::Local::<v8::Object>::try_from(value) else {
        return MouseEventData::default();
    };
    MouseEventData {
        event_type: String::new(),
        screen_x: super::event::number_property(scope, object, "screenX", 0.0) as i32,
        screen_y: super::event::number_property(scope, object, "screenY", 0.0) as i32,
        client_x: super::event::number_property(scope, object, "clientX", 0.0) as i32,
        client_y: super::event::number_property(scope, object, "clientY", 0.0) as i32,
        ctrl_key: super::event::boolean_property(scope, object, "ctrlKey"),
        shift_key: super::event::boolean_property(scope, object, "shiftKey"),
        alt_key: super::event::boolean_property(scope, object, "altKey"),
        meta_key: super::event::boolean_property(scope, object, "metaKey"),
        button: super::event::number_property(scope, object, "button", 0.0) as i16,
        buttons: super::event::number_property(scope, object, "buttons", 0.0) as u16,
        related_target: value_property(scope, object, "relatedTarget"),
        movement_x: super::event::number_property(scope, object, "movementX", 0.0) as i32,
        movement_y: super::event::number_property(scope, object, "movementY", 0.0) as i32,
        bubbles: super::event::boolean_property(scope, object, "bubbles"),
        cancelable: super::event::boolean_property(scope, object, "cancelable"),
        composed: super::event::boolean_property(scope, object, "composed"),
        view: value_property(scope, object, "view"),
        detail: super::event::number_property(scope, object, "detail", 0.0) as i32,
    }
}

pub(crate) fn attach(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    event_type: String,
    mut data: MouseEventData,
) {
    data.event_type = event_type.clone();
    if matches!(
        event_type.as_str(),
        "mousemove" | "mouseover" | "mouseout" | "mouseenter" | "mouseleave"
    ) {
        data.button = 0;
    }
    super::ui_event::attach(
        scope,
        object,
        event_type,
        data.bubbles,
        data.cancelable,
        data.composed,
        data.view.clone(),
        data.detail,
        None,
    );
    if let Some(store) = scope.get_slot_mut::<MouseEventStore>() {
        store.records.insert(object.get_identity_hash().get(), data);
    }
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    event_type: String,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let event = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, event, prototype.into()) != Some(true) {
        return Err("cannot create MouseEvent".to_owned());
    }
    attach(scope, event, event_type, MouseEventData::default());
    Ok(event)
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<MouseEventData> {
    scope
        .get_slot::<MouseEventStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn return_number(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&MouseEventData) -> i32,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new(scope, select(&record)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn return_boolean(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&MouseEventData) -> bool,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, select(&record)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_screen_x(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.screen_x);
}
pub(crate) fn get_screen_y(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.screen_y);
}
pub(crate) fn get_client_x(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.client_x);
}
pub(crate) fn get_client_y(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.client_y);
}
pub(crate) fn get_ctrl_key(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_boolean(s, a, r, |v| v.ctrl_key);
}
pub(crate) fn get_shift_key(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_boolean(s, a, r, |v| v.shift_key);
}
pub(crate) fn get_alt_key(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_boolean(s, a, r, |v| v.alt_key);
}
pub(crate) fn get_meta_key(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_boolean(s, a, r, |v| v.meta_key);
}
pub(crate) fn get_button(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| i32::from(v.button));
}
pub(crate) fn get_buttons(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| i32::from(v.buttons));
}
pub(crate) fn get_page_x(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.client_x);
}
pub(crate) fn get_page_y(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.client_y);
}
pub(crate) fn get_x(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.client_x);
}
pub(crate) fn get_y(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.client_y);
}
pub(crate) fn get_offset_x(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.client_x);
}
pub(crate) fn get_offset_y(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.client_y);
}
pub(crate) fn get_movement_x(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.movement_x);
}
pub(crate) fn get_movement_y(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.movement_y);
}
pub(crate) fn get_layer_x(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.client_x);
}
pub(crate) fn get_layer_y(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.client_y);
}

pub(crate) fn get_related_target(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(value) = record.related_target {
        result.set(v8::Local::new(scope, &value));
    } else {
        result.set(v8::null(scope).into());
    }
}

pub(crate) fn get_from_element(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let Some(record) = record(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    let mut r = r;
    if matches!(record.event_type.as_str(), "mouseover" | "mouseenter")
        && let Some(value) = record.related_target
    {
        r.set(v8::Local::new(s, &value));
    } else {
        r.set(v8::null(s).into());
    }
}

pub(crate) fn get_to_element(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if matches!(record.event_type.as_str(), "mouseout" | "mouseleave")
        && let Some(value) = record.related_target
    {
        result.set(v8::Local::new(scope, &value));
    } else {
        result.set(v8::null(scope).into());
    }
}

pub(crate) fn get_modifier_state(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let modifier = crate::webidl::value_to_string(scope, arguments.get(0));
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let active = match modifier.as_str() {
        "Control" => record.ctrl_key,
        "Shift" => record.shift_key,
        "Alt" => record.alt_key,
        "Meta" => record.meta_key,
        _ => false,
    };
    result.set(v8::Boolean::new(scope, active).into());
}

pub(crate) fn init_mouse_event(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let event_type = crate::webidl::value_to_string(scope, arguments.get(0));
    let data = MouseEventData {
        bubbles: arguments.get(1).boolean_value(scope),
        cancelable: arguments.get(2).boolean_value(scope),
        view: (!arguments.get(3).is_null() && !arguments.get(3).is_undefined())
            .then(|| v8::Global::new(scope, arguments.get(3))),
        detail: arguments.get(4).int32_value(scope).unwrap_or(0),
        screen_x: arguments.get(5).int32_value(scope).unwrap_or(0),
        screen_y: arguments.get(6).int32_value(scope).unwrap_or(0),
        client_x: arguments.get(7).int32_value(scope).unwrap_or(0),
        client_y: arguments.get(8).int32_value(scope).unwrap_or(0),
        ctrl_key: arguments.get(9).boolean_value(scope),
        alt_key: arguments.get(10).boolean_value(scope),
        shift_key: arguments.get(11).boolean_value(scope),
        meta_key: arguments.get(12).boolean_value(scope),
        button: arguments.get(13).int32_value(scope).unwrap_or(0) as i16,
        related_target: (!arguments.get(14).is_null() && !arguments.get(14).is_undefined())
            .then(|| v8::Global::new(scope, arguments.get(14))),
        ..MouseEventData::default()
    };
    attach(scope, arguments.this(), event_type, data);
}

pub(crate) fn value_property(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<v8::Global<v8::Value>> {
    let key = v8::String::new(scope, name)?;
    let value = object.get(scope, key.into())?;
    if value.is_null() || value.is_undefined() {
        None
    } else {
        Some(v8::Global::new(scope, value))
    }
}
