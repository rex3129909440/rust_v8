use std::collections::HashMap;

#[derive(Clone)]
pub(crate) struct MarqueeRecord {
    pub(crate) behavior: String,
    pub(crate) background_color: String,
    pub(crate) direction: String,
    pub(crate) height: String,
    pub(crate) horizontal_space: u32,
    pub(crate) loop_count: i32,
    pub(crate) scroll_amount: u32,
    pub(crate) scroll_delay: u32,
    pub(crate) true_speed: bool,
    pub(crate) vertical_space: u32,
    pub(crate) width: String,
    pub(crate) running: bool,
}

impl Default for MarqueeRecord {
    fn default() -> Self {
        Self {
            behavior: String::new(),
            background_color: String::new(),
            direction: String::new(),
            height: String::new(),
            horizontal_space: 0,
            loop_count: -1,
            scroll_amount: 6,
            scroll_delay: 85,
            true_speed: false,
            vertical_space: 0,
            width: String::new(),
            running: true,
        }
    }
}

#[derive(Default)]
pub(crate) struct HtmlMarqueeElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, MarqueeRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(HtmlMarqueeElementStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "HTMLMarqueeElement", constructor.into())
}
pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(constructor) = scope
        .get_slot::<HtmlMarqueeElementStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let parent = super::html_element::ensure_constructor(scope)?;
    let constructor = crate::webidl::create_function(
        scope,
        "HTMLMarqueeElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::html_marquee_element_behavior_property::define(scope, prototype)?;
    super::html_marquee_element_bg_color_property::define(scope, prototype)?;
    super::html_marquee_element_direction_property::define(scope, prototype)?;
    super::html_marquee_element_height_property::define(scope, prototype)?;
    super::html_marquee_element_hspace_property::define(scope, prototype)?;
    super::html_marquee_element_loop_property::define(scope, prototype)?;
    super::html_marquee_element_scroll_amount_property::define(scope, prototype)?;
    super::html_marquee_element_scroll_delay_property::define(scope, prototype)?;
    super::html_marquee_element_true_speed_property::define(scope, prototype)?;
    super::html_marquee_element_vspace_property::define(scope, prototype)?;
    super::html_marquee_element_width_property::define(scope, prototype)?;
    super::html_marquee_element_start::define(scope, prototype)?;
    super::html_marquee_element_stop::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<HtmlMarqueeElementStore>()
        .ok_or_else(|| "HTMLMarqueeElement state was not prepared".to_owned())?
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
        return Err("cannot create HTMLMarqueeElement".to_owned());
    }
    super::html_element::attach(scope, object, "MARQUEE");
    scope
        .get_slot_mut::<HtmlMarqueeElementStore>()
        .ok_or_else(|| "HTMLMarqueeElement state was not prepared".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), MarqueeRecord::default());
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
) -> Option<MarqueeRecord> {
    scope
        .get_slot::<HtmlMarqueeElementStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}
pub(crate) fn update(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut MarqueeRecord),
) {
    if let Some(record) = scope
        .get_slot_mut::<HtmlMarqueeElementStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        change(record);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn get_string(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
    select: impl FnOnce(&MarqueeRecord) -> &str,
) {
    if let Some(record) = record(scope, a.this()) {
        if let Some(value) = v8::String::new(scope, select(&record)) {
            r.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn set_string(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    change: impl FnOnce(&mut MarqueeRecord, String),
) {
    let value = crate::webidl::value_to_string(scope, a.get(0));
    update(scope, a.this(), |record| change(record, value));
}
pub(crate) fn get_unsigned(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
    select: impl FnOnce(&MarqueeRecord) -> u32,
) {
    if let Some(record) = record(scope, a.this()) {
        r.set(v8::Integer::new_from_unsigned(scope, select(&record)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn get_behavior(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.behavior);
}
pub(crate) fn set_behavior(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.behavior = v);
}
pub(crate) fn get_bg_color(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.background_color);
}
pub(crate) fn set_bg_color(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.background_color = v);
}
pub(crate) fn get_direction(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.direction);
}
pub(crate) fn set_direction(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.direction = v);
}
pub(crate) fn get_height(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.height);
}
pub(crate) fn set_height(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.height = v);
}
pub(crate) fn get_width(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.width);
}
pub(crate) fn set_width(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.width = v);
}
pub(crate) fn get_hspace(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_unsigned(s, a, r, |x| x.horizontal_space);
}
pub(crate) fn set_hspace(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = a.get(0).uint32_value(s).unwrap_or(0);
    update(s, a.this(), |x| x.horizontal_space = v);
}
pub(crate) fn get_loop(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, a.this()) {
        r.set(v8::Integer::new(scope, record.loop_count).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn set_loop(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = a.get(0).int32_value(s).unwrap_or(-1);
    update(s, a.this(), |x| x.loop_count = v);
}
pub(crate) fn get_scroll_amount(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_unsigned(s, a, r, |x| x.scroll_amount);
}
pub(crate) fn set_scroll_amount(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = a.get(0).uint32_value(s).unwrap_or(0);
    update(s, a.this(), |x| x.scroll_amount = v);
}
pub(crate) fn get_scroll_delay(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_unsigned(s, a, r, |x| x.scroll_delay);
}
pub(crate) fn set_scroll_delay(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = a.get(0).uint32_value(s).unwrap_or(0);
    update(s, a.this(), |x| x.scroll_delay = v);
}
pub(crate) fn get_true_speed(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, a.this()) {
        r.set(v8::Boolean::new(scope, record.true_speed).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn set_true_speed(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = a.get(0).boolean_value(s);
    update(s, a.this(), |x| x.true_speed = v);
}
pub(crate) fn get_vspace(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_unsigned(s, a, r, |x| x.vertical_space);
}
pub(crate) fn set_vspace(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = a.get(0).uint32_value(s).unwrap_or(0);
    update(s, a.this(), |x| x.vertical_space = v);
}
pub(crate) fn start(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    update(scope, a.this(), |x| x.running = true);
}
pub(crate) fn stop(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    update(scope, a.this(), |x| x.running = false);
}
