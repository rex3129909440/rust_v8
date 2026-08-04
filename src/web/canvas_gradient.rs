use std::collections::HashMap;

#[derive(Clone)]
pub(crate) enum CanvasGradientKind {
    Linear([f64; 4]),
    Radial([f64; 6]),
    Conic([f64; 3]),
}

#[derive(Default)]
pub(crate) struct CanvasGradientStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, CanvasGradientRecord>,
}

#[derive(Clone)]
struct CanvasGradientRecord {
    kind: CanvasGradientKind,
    stops: Vec<(f64, String)>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CanvasGradientStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "CanvasGradient", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<CanvasGradientStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "CanvasGradient",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_method(scope, prototype, "addColorStop", 2, add_color_stop)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<CanvasGradientStore>()
        .ok_or_else(|| "CanvasGradient state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    kind: CanvasGradientKind,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let gradient = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, gradient, prototype.into()) != Some(true) {
        return Err("cannot create CanvasGradient".to_owned());
    }
    scope
        .get_slot_mut::<CanvasGradientStore>()
        .ok_or_else(|| "CanvasGradient state was not prepared".to_owned())?
        .records
        .insert(
            gradient.get_identity_hash().get(),
            CanvasGradientRecord {
                kind,
                stops: Vec::new(),
            },
        );
    Ok(gradient)
}

pub(crate) fn is_gradient(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> bool {
    scope
        .get_slot::<CanvasGradientStore>()
        .is_some_and(|store| {
            store
                .records
                .contains_key(&object.get_identity_hash().get())
        })
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'CanvasGradient': Illegal constructor",
    );
}

fn add_color_stop(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let offset = arguments.get(0).number_value(scope).unwrap_or(f64::NAN);
    if !offset.is_finite() || !(0.0..=1.0).contains(&offset) {
        if let Ok(exception) = super::dom_exception::create(
            scope,
            "The offset must be between 0 and 1".to_owned(),
            "IndexSizeError".to_owned(),
        ) {
            scope.throw_exception(exception.into());
        }
        return;
    }
    let color = crate::webidl::value_to_string(scope, arguments.get(1));
    if color.trim().is_empty() {
        if let Ok(exception) = super::dom_exception::create(
            scope,
            "The color cannot be parsed".to_owned(),
            "SyntaxError".to_owned(),
        ) {
            scope.throw_exception(exception.into());
        }
        return;
    }
    let Some(record) = scope
        .get_slot_mut::<CanvasGradientStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let _ = &record.kind;
    record.stops.push((offset, color));
    record
        .stops
        .sort_by(|left, right| left.0.total_cmp(&right.0));
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<CanvasGradientStore>() {
        store.constructor.remove(realm_id);
    }
}
