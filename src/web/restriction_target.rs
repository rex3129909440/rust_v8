use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct RestrictionTargetStore {
    constructor: crate::webidl::RealmConstructor,
    targets: HashMap<i32, v8::Global<v8::Object>>,
    source_elements: HashMap<i32, v8::Global<v8::Object>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(RestrictionTargetStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "RestrictionTarget", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(constructor) = scope
        .get_slot::<RestrictionTargetStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "RestrictionTarget",
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
        .get_slot_mut::<RestrictionTargetStore>()
        .ok_or_else(|| "RestrictionTarget state was not prepared".to_owned())?
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
        "Failed to construct 'RestrictionTarget': Illegal constructor",
    )
}

fn from_element(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'fromElement' on 'RestrictionTarget': 1 argument required, but only 0 present.",
        );
        return;
    }
    let Ok(element) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'fromElement' on 'RestrictionTarget': parameter 1 is not of type 'Element'.",
        );
        return;
    };
    let element_id = element.get_identity_hash().get();
    if let Some(existing) = scope
        .get_slot::<RestrictionTargetStore>()
        .and_then(|store| store.targets.get(&element_id))
        .cloned()
    {
        let target = v8::Local::new(scope, &existing);
        if let Ok(promise) = super::writable_stream::resolved_promise(scope, target.into()) {
            result.set(promise.into());
        }
        return;
    }
    let Ok(constructor) = ensure_constructor(scope) else {
        return;
    };
    let Ok(prototype) = crate::webidl::prototype(scope, constructor) else {
        return;
    };
    let target = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, target, prototype.into()) != Some(true) {
        return;
    }
    let target_id = target.get_identity_hash().get();
    let element_global = v8::Global::new(scope, element);
    let target_global = v8::Global::new(scope, target);
    let store = scope
        .get_slot_mut::<RestrictionTargetStore>()
        .expect("RestrictionTarget state");
    store.source_elements.insert(target_id, element_global);
    store.targets.insert(element_id, target_global);
    if let Ok(promise) = super::writable_stream::resolved_promise(scope, target.into()) {
        result.set(promise.into());
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<RestrictionTargetStore>() {
        store.constructor.remove(realm_id);
    }
}
