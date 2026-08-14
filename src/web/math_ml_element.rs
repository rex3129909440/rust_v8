use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct MathMlElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, MathMlRecord>,
}

#[derive(Clone)]
pub(crate) struct MathMlRecord {
    pub(crate) handlers: HashMap<String, v8::Global<v8::Value>>,
    pub(crate) dataset: v8::Global<v8::Object>,
    pub(crate) style: v8::Global<v8::Object>,
    pub(crate) attribute_style_map: v8::Global<v8::Object>,
    pub(crate) nonce: String,
    pub(crate) autofocus: bool,
    pub(crate) tab_index: i32,
    pub(crate) focused: bool,
    pub(crate) focus_group: String,
    pub(crate) focus_group_start: String,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(MathMlElementStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "MathMLElement", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<MathMlElementStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "MathMLElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::math_ml_element_onabort_property::define(scope, prototype)?;
    super::math_ml_element_onbeforeinput_property::define(scope, prototype)?;
    super::math_ml_element_onbeforematch_property::define(scope, prototype)?;
    super::math_ml_element_onbeforetoggle_property::define(scope, prototype)?;
    super::math_ml_element_onblur_property::define(scope, prototype)?;
    super::math_ml_element_oncancel_property::define(scope, prototype)?;
    super::math_ml_element_oncanplay_property::define(scope, prototype)?;
    super::math_ml_element_oncanplaythrough_property::define(scope, prototype)?;
    super::math_ml_element_onchange_property::define(scope, prototype)?;
    super::math_ml_element_onclick_property::define(scope, prototype)?;
    super::math_ml_element_onclose_property::define(scope, prototype)?;
    super::math_ml_element_oncommand_property::define(scope, prototype)?;
    super::math_ml_element_oncontentvisibilityautostatechange_property::define(scope, prototype)?;
    super::math_ml_element_oncontextlost_property::define(scope, prototype)?;
    super::math_ml_element_oncontextmenu_property::define(scope, prototype)?;
    super::math_ml_element_oncontextrestored_property::define(scope, prototype)?;
    super::math_ml_element_oncuechange_property::define(scope, prototype)?;
    super::math_ml_element_ondblclick_property::define(scope, prototype)?;
    super::math_ml_element_ondrag_property::define(scope, prototype)?;
    super::math_ml_element_ondragend_property::define(scope, prototype)?;
    super::math_ml_element_ondragenter_property::define(scope, prototype)?;
    super::math_ml_element_ondragleave_property::define(scope, prototype)?;
    super::math_ml_element_ondragover_property::define(scope, prototype)?;
    super::math_ml_element_ondragstart_property::define(scope, prototype)?;
    super::math_ml_element_ondrop_property::define(scope, prototype)?;
    super::math_ml_element_ondurationchange_property::define(scope, prototype)?;
    super::math_ml_element_onemptied_property::define(scope, prototype)?;
    super::math_ml_element_onended_property::define(scope, prototype)?;
    super::math_ml_element_onerror_property::define(scope, prototype)?;
    super::math_ml_element_onfocus_property::define(scope, prototype)?;
    super::math_ml_element_onformdata_property::define(scope, prototype)?;
    super::math_ml_element_oninput_property::define(scope, prototype)?;
    super::math_ml_element_oninvalid_property::define(scope, prototype)?;
    super::math_ml_element_onkeydown_property::define(scope, prototype)?;
    super::math_ml_element_onkeypress_property::define(scope, prototype)?;
    super::math_ml_element_onkeyup_property::define(scope, prototype)?;
    super::math_ml_element_onload_property::define(scope, prototype)?;
    super::math_ml_element_onloadeddata_property::define(scope, prototype)?;
    super::math_ml_element_onloadedmetadata_property::define(scope, prototype)?;
    super::math_ml_element_onloadstart_property::define(scope, prototype)?;
    super::math_ml_element_onmousedown_property::define(scope, prototype)?;
    super::math_ml_element_onmouseenter_property::define(scope, prototype)?;
    super::math_ml_element_onmouseleave_property::define(scope, prototype)?;
    super::math_ml_element_onmousemove_property::define(scope, prototype)?;
    super::math_ml_element_onmouseout_property::define(scope, prototype)?;
    super::math_ml_element_onmouseover_property::define(scope, prototype)?;
    super::math_ml_element_onmouseup_property::define(scope, prototype)?;
    super::math_ml_element_onmousewheel_property::define(scope, prototype)?;
    super::math_ml_element_onpause_property::define(scope, prototype)?;
    super::math_ml_element_onplay_property::define(scope, prototype)?;
    super::math_ml_element_onplaying_property::define(scope, prototype)?;
    super::math_ml_element_onprogress_property::define(scope, prototype)?;
    super::math_ml_element_onratechange_property::define(scope, prototype)?;
    super::math_ml_element_onreset_property::define(scope, prototype)?;
    super::math_ml_element_onresize_property::define(scope, prototype)?;
    super::math_ml_element_onscroll_property::define(scope, prototype)?;
    super::math_ml_element_onscrollend_property::define(scope, prototype)?;
    super::math_ml_element_onsecuritypolicyviolation_property::define(scope, prototype)?;
    super::math_ml_element_onseeked_property::define(scope, prototype)?;
    super::math_ml_element_onseeking_property::define(scope, prototype)?;
    super::math_ml_element_onselect_property::define(scope, prototype)?;
    super::math_ml_element_onslotchange_property::define(scope, prototype)?;
    super::math_ml_element_onstalled_property::define(scope, prototype)?;
    super::math_ml_element_onsubmit_property::define(scope, prototype)?;
    super::math_ml_element_onsuspend_property::define(scope, prototype)?;
    super::math_ml_element_ontimeupdate_property::define(scope, prototype)?;
    super::math_ml_element_ontoggle_property::define(scope, prototype)?;
    super::math_ml_element_onvolumechange_property::define(scope, prototype)?;
    super::math_ml_element_onwaiting_property::define(scope, prototype)?;
    super::math_ml_element_onwebkitanimationend_property::define(scope, prototype)?;
    super::math_ml_element_onwebkitanimationiteration_property::define(scope, prototype)?;
    super::math_ml_element_onwebkitanimationstart_property::define(scope, prototype)?;
    super::math_ml_element_onwebkittransitionend_property::define(scope, prototype)?;
    super::math_ml_element_onwheel_property::define(scope, prototype)?;
    super::math_ml_element_onauxclick_property::define(scope, prototype)?;
    super::math_ml_element_ongotpointercapture_property::define(scope, prototype)?;
    super::math_ml_element_onlostpointercapture_property::define(scope, prototype)?;
    super::math_ml_element_onpointerdown_property::define(scope, prototype)?;
    super::math_ml_element_onpointermove_property::define(scope, prototype)?;
    super::math_ml_element_onpointerup_property::define(scope, prototype)?;
    super::math_ml_element_onpointercancel_property::define(scope, prototype)?;
    super::math_ml_element_onpointerover_property::define(scope, prototype)?;
    super::math_ml_element_onpointerout_property::define(scope, prototype)?;
    super::math_ml_element_onpointerenter_property::define(scope, prototype)?;
    super::math_ml_element_onpointerleave_property::define(scope, prototype)?;
    super::math_ml_element_onselectstart_property::define(scope, prototype)?;
    super::math_ml_element_onselectionchange_property::define(scope, prototype)?;
    super::math_ml_element_onanimationcancel_property::define(scope, prototype)?;
    super::math_ml_element_onanimationend_property::define(scope, prototype)?;
    super::math_ml_element_onanimationiteration_property::define(scope, prototype)?;
    super::math_ml_element_onanimationstart_property::define(scope, prototype)?;
    super::math_ml_element_ontransitionrun_property::define(scope, prototype)?;
    super::math_ml_element_ontransitionstart_property::define(scope, prototype)?;
    super::math_ml_element_ontransitionend_property::define(scope, prototype)?;
    super::math_ml_element_ontransitioncancel_property::define(scope, prototype)?;
    super::math_ml_element_onbeforexrselect_property::define(scope, prototype)?;
    super::math_ml_element_oncopy_property::define(scope, prototype)?;
    super::math_ml_element_oncut_property::define(scope, prototype)?;
    super::math_ml_element_onpaste_property::define(scope, prototype)?;
    super::math_ml_element_dataset_property::define(scope, prototype)?;
    super::math_ml_element_nonce_property::define(scope, prototype)?;
    super::math_ml_element_autofocus_property::define(scope, prototype)?;
    super::math_ml_element_tab_index_property::define(scope, prototype)?;
    super::math_ml_element_style_property::define(scope, prototype)?;
    super::math_ml_element_attribute_style_map_property::define(scope, prototype)?;
    super::math_ml_element_blur::define(scope, prototype)?;
    super::math_ml_element_focus::define(scope, prototype)?;
    super::math_ml_element_onscrollsnapchange_property::define(scope, prototype)?;
    super::math_ml_element_onscrollsnapchanging_property::define(scope, prototype)?;
    super::math_ml_element_focus_group_property::define(scope, prototype)?;
    super::math_ml_element_focus_group_start_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    super::math_ml_element_onpointerrawupdate_property::define(scope, prototype)?;
    for name in ["ontouchcancel", "ontouchend", "ontouchmove", "ontouchstart"] {
        super::math_ml_element_touch_handlers::define(scope, prototype, name)?;
    }
    let parent = super::element::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<MathMlElementStore>()
        .ok_or_else(|| "MathMLElement state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'MathMLElement': Illegal constructor",
    );
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    tag_name: String,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create MathMLElement".to_owned());
    }
    super::element::attach(
        scope,
        object,
        tag_name,
        Some("http://www.w3.org/1998/Math/MathML".to_owned()),
    );
    let dataset = tagged_object(scope, "DOMStringMap");
    let attribute_style_map =
        super::style_property_map::create(scope).expect("StylePropertyMap must be available");
    let style = super::css_style_declaration::create(scope, "", None, Some(attribute_style_map))
        .expect("CSSStyleDeclaration must be available");
    let dataset = v8::Global::new(scope, dataset);
    let style = v8::Global::new(scope, style);
    let attribute_style_map = v8::Global::new(scope, attribute_style_map);
    scope
        .get_slot_mut::<MathMlElementStore>()
        .ok_or_else(|| "MathMLElement state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            MathMlRecord {
                handlers: HashMap::new(),
                dataset,
                style,
                attribute_style_map,
                nonce: String::new(),
                autofocus: false,
                tab_index: -1,
                focused: false,
                focus_group: String::new(),
                focus_group_start: String::new(),
            },
        );
    Ok(object)
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<MathMlRecord> {
    scope
        .get_slot::<MathMlElementStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn clear_focus(scope: &mut v8::PinScope<'_, '_>) {
    if let Some(store) = scope.get_slot_mut::<MathMlElementStore>() {
        for record in store.records.values_mut() {
            record.focused = false;
        }
    }
}

pub(crate) fn return_object(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&MathMlRecord) -> &v8::Global<v8::Object>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, select(&record)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_dataset(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_object(s, a, r, |x| &x.dataset);
}
pub(crate) fn get_style(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_object(s, a, r, |x| &x.style);
}
pub(crate) fn get_attribute_style_map(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_object(s, a, r, |x| &x.attribute_style_map);
}

pub(crate) fn set_style(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let style = v8::Local::new(scope, &record.style);
    if let Some(key) = v8::String::new(scope, "cssText")
        && let Some(value) = arguments.get(0).to_string(scope)
    {
        let _ = style.set(scope, key.into(), value.into());
    }
}

pub(crate) fn return_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&MathMlRecord) -> &str,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, select(&record)) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    update: impl FnOnce(&mut MathMlRecord, String),
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    if let Some(record) = scope
        .get_slot_mut::<MathMlElementStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        update(record, value);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_nonce(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |x| &x.nonce);
}
pub(crate) fn set_nonce(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.nonce = v);
}
pub(crate) fn get_focus_group(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |x| &x.focus_group);
}
pub(crate) fn set_focus_group(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.focus_group = v);
}
pub(crate) fn get_focus_group_start(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |x| &x.focus_group_start);
}
pub(crate) fn set_focus_group_start(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.focus_group_start = v);
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
    if let Some(record) = scope
        .get_slot_mut::<MathMlElementStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
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
    if let Some(record) = scope
        .get_slot_mut::<MathMlElementStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.tab_index = value;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn blur(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn focus(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn tagged_object<'s>(
    scope: &v8::PinScope<'s, '_>,
    name: &str,
) -> v8::Local<'s, v8::Object> {
    let object = v8::Object::new(scope);
    let tag = v8::Symbol::get_to_string_tag(scope);
    if let Some(value) = v8::String::new(scope, name) {
        let _ = object.define_own_property(
            scope,
            tag.into(),
            value.into(),
            v8::PropertyAttribute::READ_ONLY | v8::PropertyAttribute::DONT_ENUM,
        );
    }
    object
}
