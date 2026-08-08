use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct TextMetricsStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, TextMetricsRecord>,
}

#[derive(Clone, Default)]
pub(crate) struct TextMetricsRecord {
    pub width: f64,
    pub actual_bounding_box_left: f64,
    pub actual_bounding_box_right: f64,
    pub font_bounding_box_ascent: f64,
    pub font_bounding_box_descent: f64,
    pub actual_bounding_box_ascent: f64,
    pub actual_bounding_box_descent: f64,
    pub hanging_baseline: f64,
    pub alphabetic_baseline: f64,
    pub ideographic_baseline: f64,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(TextMetricsStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "TextMetrics", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<TextMetricsStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "TextMetrics",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "width", get_width)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "actualBoundingBoxLeft",
        get_actual_bounding_box_left,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "actualBoundingBoxRight",
        get_actual_bounding_box_right,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "fontBoundingBoxAscent",
        get_font_bounding_box_ascent,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "fontBoundingBoxDescent",
        get_font_bounding_box_descent,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "actualBoundingBoxAscent",
        get_actual_bounding_box_ascent,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "actualBoundingBoxDescent",
        get_actual_bounding_box_descent,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "hangingBaseline",
        get_hanging_baseline,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "alphabeticBaseline",
        get_alphabetic_baseline,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "ideographicBaseline",
        get_ideographic_baseline,
    )?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<TextMetricsStore>()
        .ok_or_else(|| "TextMetrics state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    metrics: TextMetricsRecord,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create TextMetrics".to_owned());
    }
    scope
        .get_slot_mut::<TextMetricsStore>()
        .ok_or_else(|| "TextMetrics state was not prepared".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), metrics);
    Ok(object)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'TextMetrics': Illegal constructor",
    );
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<TextMetricsRecord> {
    scope
        .get_slot::<TextMetricsStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn return_number(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&TextMetricsRecord) -> f64,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, select(&record)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_width(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.width);
}
fn get_actual_bounding_box_left(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.actual_bounding_box_left);
}
fn get_actual_bounding_box_right(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.actual_bounding_box_right);
}
fn get_font_bounding_box_ascent(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.font_bounding_box_ascent);
}
fn get_font_bounding_box_descent(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.font_bounding_box_descent);
}
fn get_actual_bounding_box_ascent(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.actual_bounding_box_ascent);
}
fn get_actual_bounding_box_descent(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.actual_bounding_box_descent);
}
fn get_hanging_baseline(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.hanging_baseline);
}
fn get_alphabetic_baseline(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.alphabetic_baseline);
}
fn get_ideographic_baseline(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.ideographic_baseline);
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<TextMetricsStore>() {
        store.constructor.remove(realm_id);
    }
}
