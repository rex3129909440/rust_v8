use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct CropTargetStore {
    constructor: crate::webidl::RealmConstructor,
    targets: HashMap<i32, v8::Global<v8::Object>>,
    elements: HashMap<i32, v8::Global<v8::Object>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CropTargetStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "CropTarget", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(constructor) = scope
        .get_slot::<CropTargetStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "CropTarget",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::define_method(scope, constructor.into(), "fromElement", 1, from_element)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<CropTargetStore>()
        .ok_or_else(|| "CropTarget state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'CropTarget': Illegal constructor",
    );
}

fn from_element(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'fromElement' on 'CropTarget': 1 argument required, but only 0 present.",
        );
        return;
    }
    let Ok(element) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        throw_element_type(scope);
        return;
    };
    if super::element::record(scope, element).is_none() {
        throw_element_type(scope);
        return;
    }
    let identity = element.get_identity_hash().get();
    let existing = scope
        .get_slot::<CropTargetStore>()
        .and_then(|store| store.targets.get(&identity))
        .cloned();
    let target = if let Some(target) = existing {
        v8::Local::new(scope, &target)
    } else {
        let Ok(target) = create(scope, element) else {
            return;
        };
        target
    };
    if let Ok(promise) = super::writable_stream::resolved_promise(scope, target.into()) {
        result.set(promise.into());
    }
}

fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    element: v8::Local<'_, v8::Object>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let target = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, target, prototype.into()) != Some(true) {
        return Err("cannot create CropTarget".to_owned());
    }
    let element_identity = element.get_identity_hash().get();
    let target_identity = target.get_identity_hash().get();
    let target_global = v8::Global::new(scope, target);
    let element_global = v8::Global::new(scope, element);
    let store = scope
        .get_slot_mut::<CropTargetStore>()
        .ok_or_else(|| "CropTarget state was not prepared".to_owned())?;
    store.targets.insert(element_identity, target_global);
    store.elements.insert(target_identity, element_global);
    Ok(target)
}

fn throw_element_type(scope: &mut v8::PinScope<'_, '_>) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to execute 'fromElement' on 'CropTarget': parameter 1 is not of type 'Element'.",
    );
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<CropTargetStore>() {
        store.constructor.remove(realm_id);
    }
}
