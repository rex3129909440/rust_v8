use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct SvgAnimatedAngleStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, Record>,
}

#[derive(Clone)]
struct Record {
    base: v8::Global<v8::Object>,
    animated: v8::Global<v8::Object>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(SvgAnimatedAngleStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "SVGAnimatedAngle", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<SvgAnimatedAngleStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "SVGAnimatedAngle",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "baseVal", get_base)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "animVal", get_animated)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<SvgAnimatedAngleStore>()
        .ok_or_else(|| "SVGAnimatedAngle state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: super::svg_angle::AngleSnapshot,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create SVGAnimatedAngle".to_owned());
    }
    let base = super::svg_angle::create_from(scope, value)?;
    let animated = super::svg_angle::create_from(scope, value)?;
    let record = Record {
        base: v8::Global::new(scope, base),
        animated: v8::Global::new(scope, animated),
    };
    scope
        .get_slot_mut::<SvgAnimatedAngleStore>()
        .ok_or_else(|| "SVGAnimatedAngle state was not prepared".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), record);
    Ok(object)
}

pub(crate) fn set(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    value: super::svg_angle::AngleSnapshot,
) -> Result<(), String> {
    let base = super::svg_angle::create_from(scope, value)?;
    let animated = super::svg_angle::create_from(scope, value)?;
    let replacement = Record {
        base: v8::Global::new(scope, base),
        animated: v8::Global::new(scope, animated),
    };
    let record = scope
        .get_slot_mut::<SvgAnimatedAngleStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
        .ok_or_else(|| "Illegal invocation".to_owned())?;
    *record = replacement;
    Ok(())
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'SVGAnimatedAngle': Illegal constructor",
    );
}

fn record(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> Option<Record> {
    scope
        .get_slot::<SvgAnimatedAngleStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_base(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.base).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_animated(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.animated).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
