use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct CanvasPatternStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, CanvasPatternRecord>,
}

#[derive(Clone)]
struct CanvasPatternRecord {
    source: v8::Global<v8::Object>,
    repetition: String,
    transform: [f64; 6],
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CanvasPatternStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "CanvasPattern", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<CanvasPatternStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "CanvasPattern",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_method(scope, prototype, "setTransform", 0, set_transform)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<CanvasPatternStore>()
        .ok_or_else(|| "CanvasPattern state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    source: v8::Local<'_, v8::Object>,
    repetition: &str,
) -> Result<v8::Local<'s, v8::Object>, String> {
    if !matches!(repetition, "repeat" | "repeat-x" | "repeat-y" | "no-repeat") {
        return Err("The repetition type is invalid".to_owned());
    }
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let pattern = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, pattern, prototype.into()) != Some(true) {
        return Err("cannot create CanvasPattern".to_owned());
    }
    let source = v8::Global::new(scope, source);
    scope
        .get_slot_mut::<CanvasPatternStore>()
        .ok_or_else(|| "CanvasPattern state was not prepared".to_owned())?
        .records
        .insert(
            pattern.get_identity_hash().get(),
            CanvasPatternRecord {
                source,
                repetition: repetition.to_owned(),
                transform: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            },
        );
    Ok(pattern)
}

pub(crate) fn is_pattern(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> bool {
    scope.get_slot::<CanvasPatternStore>().is_some_and(|store| {
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
        "Failed to construct 'CanvasPattern': Illegal constructor",
    );
}

fn set_transform(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if !is_pattern(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let matrix = if arguments.get(0).is_undefined() {
        [1.0, 0.0, 0.0, 1.0, 0.0, 0.0]
    } else {
        let Ok(object) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
            crate::webidl::throw_type_error(scope, "The transform must be a matrix");
            return;
        };
        [
            super::event::number_property(scope, object, "a", 1.0),
            super::event::number_property(scope, object, "b", 0.0),
            super::event::number_property(scope, object, "c", 0.0),
            super::event::number_property(scope, object, "d", 1.0),
            super::event::number_property(scope, object, "e", 0.0),
            super::event::number_property(scope, object, "f", 0.0),
        ]
    };
    if let Some(record) = scope
        .get_slot_mut::<CanvasPatternStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        let _ = (&record.source, &record.repetition);
        record.transform = matrix;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<CanvasPatternStore>() {
        store.constructor.remove(realm_id);
    }
}
