use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct SvgElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, SvgElementRecord>,
}

#[derive(Clone)]
pub(crate) struct SvgElementRecord {
    pub(crate) class_name: v8::Global<v8::Object>,
    pub(crate) owner_svg_element: Option<v8::Global<v8::Object>>,
    pub(crate) viewport_element: Option<v8::Global<v8::Object>>,
    pub(crate) handlers: HashMap<String, v8::Global<v8::Value>>,
    pub(crate) dataset: v8::Global<v8::Object>,
    pub(crate) nonce: String,
    pub(crate) autofocus: bool,
    pub(crate) tab_index: i32,
    pub(crate) style: v8::Global<v8::Object>,
    pub(crate) attribute_style_map: v8::Global<v8::Object>,
    pub(crate) focused: bool,
    pub(crate) focus_group: String,
    pub(crate) focus_group_start: String,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(SvgElementStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "SVGElement", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<SvgElementStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "SVGElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::svg_element_class_name_property::define(scope, prototype)?;
    super::svg_element_owner_svgelement_property::define(scope, prototype)?;
    super::svg_element_viewport_element_property::define(scope, prototype)?;
    super::svg_element_onabort_property::define(scope, prototype)?;
    super::svg_element_onbeforeinput_property::define(scope, prototype)?;
    super::svg_element_onbeforematch_property::define(scope, prototype)?;
    super::svg_element_onbeforetoggle_property::define(scope, prototype)?;
    super::svg_element_onblur_property::define(scope, prototype)?;
    super::svg_element_oncancel_property::define(scope, prototype)?;
    super::svg_element_oncanplay_property::define(scope, prototype)?;
    super::svg_element_oncanplaythrough_property::define(scope, prototype)?;
    super::svg_element_onchange_property::define(scope, prototype)?;
    super::svg_element_onclick_property::define(scope, prototype)?;
    super::svg_element_onclose_property::define(scope, prototype)?;
    super::svg_element_oncommand_property::define(scope, prototype)?;
    super::svg_element_oncontentvisibilityautostatechange_property::define(scope, prototype)?;
    super::svg_element_oncontextlost_property::define(scope, prototype)?;
    super::svg_element_oncontextmenu_property::define(scope, prototype)?;
    super::svg_element_oncontextrestored_property::define(scope, prototype)?;
    super::svg_element_oncuechange_property::define(scope, prototype)?;
    super::svg_element_ondblclick_property::define(scope, prototype)?;
    super::svg_element_ondrag_property::define(scope, prototype)?;
    super::svg_element_ondragend_property::define(scope, prototype)?;
    super::svg_element_ondragenter_property::define(scope, prototype)?;
    super::svg_element_ondragleave_property::define(scope, prototype)?;
    super::svg_element_ondragover_property::define(scope, prototype)?;
    super::svg_element_ondragstart_property::define(scope, prototype)?;
    super::svg_element_ondrop_property::define(scope, prototype)?;
    super::svg_element_ondurationchange_property::define(scope, prototype)?;
    super::svg_element_onemptied_property::define(scope, prototype)?;
    super::svg_element_onended_property::define(scope, prototype)?;
    super::svg_element_onerror_property::define(scope, prototype)?;
    super::svg_element_onfocus_property::define(scope, prototype)?;
    super::svg_element_onformdata_property::define(scope, prototype)?;
    super::svg_element_oninput_property::define(scope, prototype)?;
    super::svg_element_oninvalid_property::define(scope, prototype)?;
    super::svg_element_onkeydown_property::define(scope, prototype)?;
    super::svg_element_onkeypress_property::define(scope, prototype)?;
    super::svg_element_onkeyup_property::define(scope, prototype)?;
    super::svg_element_onload_property::define(scope, prototype)?;
    super::svg_element_onloadeddata_property::define(scope, prototype)?;
    super::svg_element_onloadedmetadata_property::define(scope, prototype)?;
    super::svg_element_onloadstart_property::define(scope, prototype)?;
    super::svg_element_onmousedown_property::define(scope, prototype)?;
    super::svg_element_onmouseenter_property::define(scope, prototype)?;
    super::svg_element_onmouseleave_property::define(scope, prototype)?;
    super::svg_element_onmousemove_property::define(scope, prototype)?;
    super::svg_element_onmouseout_property::define(scope, prototype)?;
    super::svg_element_onmouseover_property::define(scope, prototype)?;
    super::svg_element_onmouseup_property::define(scope, prototype)?;
    super::svg_element_onmousewheel_property::define(scope, prototype)?;
    super::svg_element_onpause_property::define(scope, prototype)?;
    super::svg_element_onplay_property::define(scope, prototype)?;
    super::svg_element_onplaying_property::define(scope, prototype)?;
    super::svg_element_onprogress_property::define(scope, prototype)?;
    super::svg_element_onratechange_property::define(scope, prototype)?;
    super::svg_element_onreset_property::define(scope, prototype)?;
    super::svg_element_onresize_property::define(scope, prototype)?;
    super::svg_element_onscroll_property::define(scope, prototype)?;
    super::svg_element_onscrollend_property::define(scope, prototype)?;
    super::svg_element_onsecuritypolicyviolation_property::define(scope, prototype)?;
    super::svg_element_onseeked_property::define(scope, prototype)?;
    super::svg_element_onseeking_property::define(scope, prototype)?;
    super::svg_element_onselect_property::define(scope, prototype)?;
    super::svg_element_onslotchange_property::define(scope, prototype)?;
    super::svg_element_onstalled_property::define(scope, prototype)?;
    super::svg_element_onsubmit_property::define(scope, prototype)?;
    super::svg_element_onsuspend_property::define(scope, prototype)?;
    super::svg_element_ontimeupdate_property::define(scope, prototype)?;
    super::svg_element_ontoggle_property::define(scope, prototype)?;
    super::svg_element_onvolumechange_property::define(scope, prototype)?;
    super::svg_element_onwaiting_property::define(scope, prototype)?;
    super::svg_element_onwebkitanimationend_property::define(scope, prototype)?;
    super::svg_element_onwebkitanimationiteration_property::define(scope, prototype)?;
    super::svg_element_onwebkitanimationstart_property::define(scope, prototype)?;
    super::svg_element_onwebkittransitionend_property::define(scope, prototype)?;
    super::svg_element_onwheel_property::define(scope, prototype)?;
    super::svg_element_onauxclick_property::define(scope, prototype)?;
    super::svg_element_ongotpointercapture_property::define(scope, prototype)?;
    super::svg_element_onlostpointercapture_property::define(scope, prototype)?;
    super::svg_element_onpointerdown_property::define(scope, prototype)?;
    super::svg_element_onpointermove_property::define(scope, prototype)?;
    super::svg_element_onpointerup_property::define(scope, prototype)?;
    super::svg_element_onpointercancel_property::define(scope, prototype)?;
    super::svg_element_onpointerover_property::define(scope, prototype)?;
    super::svg_element_onpointerout_property::define(scope, prototype)?;
    super::svg_element_onpointerenter_property::define(scope, prototype)?;
    super::svg_element_onpointerleave_property::define(scope, prototype)?;
    super::svg_element_onselectstart_property::define(scope, prototype)?;
    super::svg_element_onselectionchange_property::define(scope, prototype)?;
    super::svg_element_onanimationcancel_property::define(scope, prototype)?;
    super::svg_element_onanimationend_property::define(scope, prototype)?;
    super::svg_element_onanimationiteration_property::define(scope, prototype)?;
    super::svg_element_onanimationstart_property::define(scope, prototype)?;
    super::svg_element_ontransitionrun_property::define(scope, prototype)?;
    super::svg_element_ontransitionstart_property::define(scope, prototype)?;
    super::svg_element_ontransitionend_property::define(scope, prototype)?;
    super::svg_element_ontransitioncancel_property::define(scope, prototype)?;
    super::svg_element_onbeforexrselect_property::define(scope, prototype)?;
    super::svg_element_oncopy_property::define(scope, prototype)?;
    super::svg_element_oncut_property::define(scope, prototype)?;
    super::svg_element_onpaste_property::define(scope, prototype)?;
    super::svg_element_dataset_property::define(scope, prototype)?;
    super::svg_element_nonce_property::define(scope, prototype)?;
    super::svg_element_autofocus_property::define(scope, prototype)?;
    super::svg_element_tab_index_property::define(scope, prototype)?;
    super::svg_element_style_property::define(scope, prototype)?;
    super::svg_element_attribute_style_map_property::define(scope, prototype)?;
    super::svg_element_blur::define(scope, prototype)?;
    super::svg_element_focus::define(scope, prototype)?;
    super::svg_element_onscrollsnapchange_property::define(scope, prototype)?;
    super::svg_element_onscrollsnapchanging_property::define(scope, prototype)?;
    super::svg_element_focus_group_property::define(scope, prototype)?;
    super::svg_element_focus_group_start_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    super::svg_element_onpointerrawupdate_property::define(scope, prototype)?;
    for name in ["ontouchcancel", "ontouchend", "ontouchmove", "ontouchstart"] {
        super::svg_element_touch_handlers::define(scope, prototype, name)?;
    }
    let parent = super::element::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<SvgElementStore>()
        .ok_or_else(|| "SVGElement state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create_with_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    constructor: v8::Local<'s, v8::Function>,
    tag_name: &str,
    owner_svg_element: Option<v8::Local<'s, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err(format!("cannot create {tag_name} SVG element"));
    }
    attach(scope, object, tag_name, owner_svg_element)?;
    Ok(object)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    tag_name: &str,
    owner_svg_element: Option<v8::Local<'s, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    create_with_constructor(scope, constructor, tag_name, owner_svg_element)
}

pub(crate) fn attach(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    tag_name: &str,
    owner_svg_element: Option<v8::Local<'_, v8::Object>>,
) -> Result<(), String> {
    super::element::attach(
        scope,
        object,
        tag_name.to_owned(),
        Some("http://www.w3.org/2000/svg".to_owned()),
    );
    let class_name = super::svg_animated_string::create(scope, "")?;
    let dataset = v8::Object::new(scope);
    let style = v8::Object::new(scope);
    let css_text_key = crate::webidl::string(scope, "cssText")?;
    let empty = crate::webidl::string(scope, "")?;
    let _ = style.create_data_property(scope, css_text_key.into(), empty.into());
    let attribute_style_map = super::style_property_map::create(scope)?;
    let record = SvgElementRecord {
        class_name: v8::Global::new(scope, class_name),
        owner_svg_element: owner_svg_element.map(|value| v8::Global::new(scope, value)),
        viewport_element: owner_svg_element.map(|value| v8::Global::new(scope, value)),
        handlers: HashMap::new(),
        dataset: v8::Global::new(scope, dataset),
        nonce: String::new(),
        autofocus: false,
        tab_index: -1,
        style: v8::Global::new(scope, style),
        attribute_style_map: v8::Global::new(scope, attribute_style_map),
        focused: false,
        focus_group: String::new(),
        focus_group_start: String::new(),
    };
    scope
        .get_slot_mut::<SvgElementStore>()
        .ok_or_else(|| "SVGElement state was not prepared".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), record);
    Ok(())
}

pub(crate) fn dispatch_handler(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    event: v8::Local<'_, v8::Object>,
    event_type: &str,
) {
    let name = format!("on{event_type}");
    let handler = record(scope, target).and_then(|record| record.handlers.get(&name).cloned());
    let Some(handler) = handler else {
        return;
    };
    let value = v8::Local::new(scope, &handler);
    if let Ok(function) = v8::Local::<v8::Function>::try_from(value) {
        v8::tc_scope!(let try_catch, scope);
        let _ = function.call(try_catch, target.into(), &[event.into()]);
    }
}

pub(crate) fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'SVGElement': Illegal constructor",
    );
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<SvgElementRecord> {
    scope
        .get_slot::<SvgElementStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn return_object(
    scope: &v8::PinScope<'_, '_>,
    value: &v8::Global<v8::Object>,
    mut result: v8::ReturnValue<'_>,
) {
    result.set(v8::Local::new(scope, value).into());
}

pub(crate) fn return_string(
    scope: &v8::PinScope<'_, '_>,
    value: &str,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(value) = v8::String::new(scope, value) {
        result.set(value.into());
    }
}

pub(crate) fn get_class_name(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_object(scope, &record.class_name, result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn return_optional_object(
    scope: &mut v8::PinScope<'_, '_>,
    value: Option<v8::Global<v8::Object>>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(value) = value {
        result.set(v8::Local::new(scope, &value).into());
    } else {
        result.set(v8::null(scope).into());
    }
}

pub(crate) fn get_owner_svg_element(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_optional_object(scope, record.owner_svg_element, result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_viewport_element(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_optional_object(scope, record.viewport_element, result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_dataset(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_object(scope, &record.dataset, result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_nonce(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_string(scope, &record.nonce, result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_nonce(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    if let Some(record) = scope.get_slot_mut::<SvgElementStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) {
        record.nonce = value;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_autofocus(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, record.autofocus).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_autofocus(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0).boolean_value(scope);
    if let Some(record) = scope.get_slot_mut::<SvgElementStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) {
        record.autofocus = value;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_tab_index(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new(scope, record.tab_index).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_tab_index(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0).int32_value(scope).unwrap_or(0);
    if let Some(record) = scope.get_slot_mut::<SvgElementStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) {
        record.tab_index = value;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_style(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_object(scope, &record.style, result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_style(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let source = crate::webidl::value_to_string(scope, arguments.get(0));
    let Some(style) = record(scope, arguments.this()).map(|record| record.style) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let (Some(key), Some(value)) = (
        v8::String::new(scope, "cssText"),
        v8::String::new(scope, &source),
    ) {
        let object = v8::Local::new(scope, &style);
        let _ = object.set(scope, key.into(), value.into());
    }
}

pub(crate) fn get_attribute_style_map(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_object(scope, &record.attribute_style_map, result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn blur(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if let Some(record) = scope.get_slot_mut::<SvgElementStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) {
        record.focused = false;
        super::element::update_document_focus(scope, arguments.this(), false);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn focus(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let valid = scope.get_slot::<SvgElementStore>().is_some_and(|store| {
        store
            .records
            .contains_key(&arguments.this().get_identity_hash().get())
    });
    if !valid {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    super::html_element::clear_focus(scope);
    clear_focus(scope);
    super::math_ml_element::clear_focus(scope);
    if let Some(record) = scope.get_slot_mut::<SvgElementStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) {
        record.focused = true;
    }
    super::element::update_document_focus(scope, arguments.this(), true);
}

pub(crate) fn clear_focus(scope: &mut v8::PinScope<'_, '_>) {
    if let Some(store) = scope.get_slot_mut::<SvgElementStore>() {
        for record in store.records.values_mut() {
            record.focused = false;
        }
    }
}

pub(crate) fn get_focus_group(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_string(scope, &record.focus_group, result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_focus_group(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    if let Some(record) = scope.get_slot_mut::<SvgElementStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) {
        record.focus_group = value;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_focus_group_start(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_string(scope, &record.focus_group_start, result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_focus_group_start(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    if let Some(record) = scope.get_slot_mut::<SvgElementStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) {
        record.focus_group_start = value;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
