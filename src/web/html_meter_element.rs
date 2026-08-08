use std::collections::HashMap;

#[derive(Clone)]
pub(crate) struct MeterRecord {
    pub(crate) value: f64,
    pub(crate) min: f64,
    pub(crate) max: f64,
    pub(crate) low: f64,
    pub(crate) high: f64,
    pub(crate) optimum: f64,
    pub(crate) labels: v8::Global<v8::Object>,
}

#[derive(Default)]
pub(crate) struct HtmlMeterElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, MeterRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(HtmlMeterElementStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "HTMLMeterElement", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(constructor) = scope
        .get_slot::<HtmlMeterElementStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let parent = super::html_element::ensure_constructor(scope)?;
    let constructor = crate::webidl::create_function(
        scope,
        "HTMLMeterElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::html_meter_element_value_property::define(scope, prototype)?;
    super::html_meter_element_min_property::define(scope, prototype)?;
    super::html_meter_element_max_property::define(scope, prototype)?;
    super::html_meter_element_low_property::define(scope, prototype)?;
    super::html_meter_element_high_property::define(scope, prototype)?;
    super::html_meter_element_optimum_property::define(scope, prototype)?;
    super::html_meter_element_labels_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<HtmlMeterElementStore>()
        .ok_or_else(|| "HTMLMeterElement state was not prepared".to_owned())?
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
        return Err("cannot create HTMLMeterElement".to_owned());
    }
    super::html_element::attach(scope, object, "METER");
    let labels = super::node_list::create(scope, Vec::new())?;
    super::node_list::register_labels_owner(scope, labels, object);
    let labels = v8::Global::new(scope, labels);
    scope
        .get_slot_mut::<HtmlMeterElementStore>()
        .ok_or_else(|| "HTMLMeterElement state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            MeterRecord {
                value: 0.0,
                min: 0.0,
                max: 1.0,
                low: 0.0,
                high: 1.0,
                optimum: 0.5,
                labels,
            },
        );
    Ok(object)
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<MeterRecord> {
    scope
        .get_slot::<HtmlMeterElementStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn update(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut MeterRecord),
) {
    if let Some(record) = scope
        .get_slot_mut::<HtmlMeterElementStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        change(record);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor");
}

pub(crate) fn number_argument(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> f64 {
    value.number_value(scope).unwrap_or(0.0)
}

pub(crate) fn return_number(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&MeterRecord) -> f64,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, select(&record)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_value(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |x| x.value.clamp(x.min, x.max.max(x.min)));
}
pub(crate) fn set_value(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = number_argument(s, a.get(0));
    update(s, a.this(), |x| x.value = value);
}
pub(crate) fn get_min(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |x| x.min);
}
pub(crate) fn set_min(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = number_argument(s, a.get(0));
    update(s, a.this(), |x| x.min = value);
}
pub(crate) fn get_max(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |x| x.max.max(x.min));
}
pub(crate) fn set_max(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = number_argument(s, a.get(0));
    update(s, a.this(), |x| x.max = value);
}
pub(crate) fn get_low(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |x| x.low.clamp(x.min, x.max.max(x.min)));
}
pub(crate) fn set_low(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = number_argument(s, a.get(0));
    update(s, a.this(), |x| x.low = value);
}
pub(crate) fn get_high(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |x| {
        x.high
            .clamp(x.low.clamp(x.min, x.max.max(x.min)), x.max.max(x.min))
    });
}
pub(crate) fn set_high(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = number_argument(s, a.get(0));
    update(s, a.this(), |x| x.high = value);
}
pub(crate) fn get_optimum(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |x| x.optimum.clamp(x.min, x.max.max(x.min)));
}
pub(crate) fn set_optimum(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = number_argument(s, a.get(0));
    update(s, a.this(), |x| x.optimum = value);
}
pub(crate) fn get_labels(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.labels).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
