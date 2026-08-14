use super::html_element::*;

pub(crate) enum InputActivation {
    None,
    Checkbox {
        target: v8::Global<v8::Object>,
        checked: bool,
        checked_dirty: bool,
        indeterminate: bool,
    },
    Radio {
        changed: bool,
        states: Vec<(v8::Global<v8::Object>, bool, bool)>,
    },
}

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "click", 0, click)
}

pub(crate) fn click(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let target = arguments.this();
    if record(scope, target).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    if is_disabled(scope, target) || !begin_click_activation(scope, target) {
        return;
    }

    let activation = input_activation_before_click(scope, target);
    let allowed = create_click_event(scope)
        .map(|event| super::event_target::dispatch(scope, target, event))
        .unwrap_or(true);

    if !allowed {
        rollback_input_activation(scope, activation);
    } else {
        finish_input_activation(scope, target, &activation);
        run_activation_default(scope, target);
        if super::html_label_element::value(scope, target).is_some()
            && let Some(control) = super::html_label_element::control_for_label(scope, target)
            && control.get_identity_hash().get() != target.get_identity_hash().get()
        {
            activate_label_control(scope, control);
        }
    }
    finish_click_activation(scope, target);
}

pub(crate) fn activate_trusted(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
) -> Result<bool, String> {
    if record(scope, target).is_none() || is_disabled(scope, target) {
        return Ok(false);
    }
    if !begin_click_activation(scope, target) {
        return Ok(true);
    }
    let activation = input_activation_before_click(scope, target);
    let event = create_click_event(scope)?;
    super::event::set_trusted(scope, event, true);
    let allowed = super::event_target::dispatch(scope, target, event);
    if allowed {
        finish_input_activation(scope, target, &activation);
        run_activation_default(scope, target);
    } else {
        rollback_input_activation(scope, activation);
    }
    finish_click_activation(scope, target);
    Ok(true)
}

pub(crate) fn begin_trusted_activation(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
) -> Option<InputActivation> {
    if record(scope, target).is_none()
        || is_disabled(scope, target)
        || !begin_click_activation(scope, target)
    {
        return None;
    }
    Some(input_activation_before_click(scope, target))
}

pub(crate) fn finish_trusted_activation(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    activation: InputActivation,
    allowed: bool,
) {
    if allowed {
        finish_input_activation(scope, target, &activation);
        run_activation_default(scope, target);
    } else {
        rollback_input_activation(scope, activation);
    }
    finish_click_activation(scope, target);
}

fn create_click_event<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let view: v8::Local<v8::Value> = scope.get_current_context().global(scope).into();
    let mouse = super::mouse_event::MouseEventData {
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
        bubbles: true,
        cancelable: true,
        composed: true,
        view: Some(v8::Global::new(scope, view)),
        detail: 0,
    };
    super::pointer_event::create_with_data(
        scope,
        "click".to_owned(),
        mouse,
        super::pointer_event::PointerRecord {
            pointer_id: -1,
            width: 1.0,
            height: 1.0,
            pressure: 0.0,
            tilt_x: 0,
            tilt_y: 0,
            azimuth_angle: 0.0,
            altitude_angle: std::f64::consts::FRAC_PI_2,
            tangential_pressure: 0.0,
            twist: 0,
            pointer_type: String::new(),
            is_primary: false,
            persistent_device_id: 0,
        },
    )
}

fn input_activation_before_click(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
) -> InputActivation {
    let Some(record) = super::html_input_element::record(scope, target) else {
        return InputActivation::None;
    };
    match record.input_type.as_str() {
        "checkbox" => {
            let previous = InputActivation::Checkbox {
                target: v8::Global::new(scope, target),
                checked: record.checked,
                checked_dirty: record.checked_dirty,
                indeterminate: record.indeterminate,
            };
            super::html_input_element::update(scope, target, |current| {
                current.checked = !current.checked;
                current.checked_dirty = true;
                current.indeterminate = false;
            });
            previous
        }
        "radio" => activate_radio_before_click(scope, target, &record),
        _ => InputActivation::None,
    }
}

fn activate_radio_before_click(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    target_record: &super::html_input_element::InputRecord,
) -> InputActivation {
    let candidates = radio_group_candidates(scope, target);
    let target_form = super::html_form_element::ancestor_form(scope, target)
        .map(|form| form.get_identity_hash().get());
    let mut states = Vec::new();
    for candidate in candidates {
        let Some(record) = super::html_input_element::record(scope, candidate) else {
            continue;
        };
        let same_form = super::html_form_element::ancestor_form(scope, candidate)
            .map(|form| form.get_identity_hash().get())
            == target_form;
        let same_group = candidate.get_identity_hash().get() == target.get_identity_hash().get()
            || (!target_record.name.is_empty()
                && record.input_type == "radio"
                && record.name == target_record.name
                && same_form);
        if !same_group {
            continue;
        }
        states.push((
            v8::Global::new(scope, candidate),
            record.checked,
            record.checked_dirty,
        ));
        super::html_input_element::update(scope, candidate, |current| {
            current.checked =
                candidate.get_identity_hash().get() == target.get_identity_hash().get();
            current.checked_dirty = true;
        });
    }
    InputActivation::Radio {
        changed: !target_record.checked,
        states,
    }
}

fn radio_group_candidates<'s>(
    scope: &v8::PinScope<'s, '_>,
    target: v8::Local<'s, v8::Object>,
) -> Vec<v8::Local<'s, v8::Object>> {
    if let Some(document) = super::node::owner_document(scope, target) {
        return super::document::document_descendants(scope, document);
    }
    let mut root = target;
    while let Some(parent) = super::node::parent(scope, root) {
        root = parent;
    }
    let mut candidates = super::dom_selector::descendants(scope, root);
    candidates.insert(0, root);
    candidates
}

fn rollback_input_activation(scope: &mut v8::PinScope<'_, '_>, activation: InputActivation) {
    match activation {
        InputActivation::None => {}
        InputActivation::Checkbox {
            target,
            checked,
            checked_dirty,
            indeterminate,
        } => {
            let target = v8::Local::new(scope, &target);
            super::html_input_element::update(scope, target, |record| {
                record.checked = checked;
                record.checked_dirty = checked_dirty;
                record.indeterminate = indeterminate;
            });
        }
        InputActivation::Radio { states, .. } => {
            for (object, checked, checked_dirty) in states {
                let object = v8::Local::new(scope, &object);
                super::html_input_element::update(scope, object, |record| {
                    record.checked = checked;
                    record.checked_dirty = checked_dirty;
                });
            }
        }
    }
}

fn finish_input_activation(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    activation: &InputActivation,
) {
    let changed = matches!(activation, InputActivation::Checkbox { .. })
        || matches!(activation, InputActivation::Radio { changed: true, .. });
    if changed {
        dispatch_trusted_state_event(scope, target, "input", true);
        dispatch_trusted_state_event(scope, target, "change", false);
    }
}

fn dispatch_trusted_state_event(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    event_type: &str,
    composed: bool,
) {
    if let Ok(event) = super::event::create(scope, event_type) {
        super::event::attach(scope, event, event_type.to_owned(), true, false, composed);
        super::event::set_trusted(scope, event, true);
        let _ = super::event_target::dispatch(scope, target, event);
    }
}

fn activate_label_control(scope: &mut v8::PinScope<'_, '_>, control: v8::Local<'_, v8::Object>) {
    if is_disabled(scope, control) || !begin_click_activation(scope, control) {
        return;
    }
    let activation = input_activation_before_click(scope, control);
    let allowed = create_click_event(scope)
        .map(|event| super::event_target::dispatch(scope, control, event))
        .unwrap_or(true);
    if allowed {
        finish_input_activation(scope, control, &activation);
    } else {
        rollback_input_activation(scope, activation);
    }
    finish_click_activation(scope, control);
}

fn is_disabled(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> bool {
    super::element::attribute_value(scope, object, "disabled").is_some()
        || super::html_button_element::record(scope, object).is_some_and(|record| record.disabled)
        || super::html_input_element::record(scope, object).is_some_and(|record| record.disabled)
        || super::html_select_element::record(scope, object).is_some_and(|record| record.disabled)
        || super::html_text_area_element::record(scope, object)
            .is_some_and(|record| record.booleans.get("disabled").copied().unwrap_or(false))
}

fn run_activation_default(scope: &mut v8::PinScope<'_, '_>, target: v8::Local<'_, v8::Object>) {
    if let Some(record) = super::html_button_element::record(scope, target)
        && let Some(form) = super::html_form_element::ancestor_form(scope, target)
    {
        match record.button_type.as_str() {
            "reset" => super::html_form_element::reset_from_activation(scope, form),
            "submit" => super::html_form_element::submit_from_activation(scope, form, Some(target)),
            _ => {}
        }
        return;
    }
    if let Some(record) = super::html_input_element::record(scope, target)
        && let Some(form) = super::html_form_element::ancestor_form(scope, target)
    {
        match record.input_type.as_str() {
            "reset" => super::html_form_element::reset_from_activation(scope, form),
            "submit" | "image" => {
                super::html_form_element::submit_from_activation(scope, form, Some(target))
            }
            _ => {}
        }
        return;
    }
    if super::html_anchor_element::record(scope, target).is_none() {
        return;
    }
    if super::element::attribute_value(scope, target, "download").is_some()
        || super::element::attribute_value(scope, target, "href").is_none()
    {
        return;
    }
    let Some(href) = super::element::resolved_url_attribute(scope, target, "href") else {
        return;
    };
    let global = scope.get_current_context().global(scope);
    let Some(location_key) = v8::String::new(scope, "location") else {
        return;
    };
    let Some(href) = v8::String::new(scope, &href) else {
        return;
    };
    let _ = global.set(scope, location_key.into(), href.into());
}
