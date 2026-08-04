use std::collections::HashMap;

#[derive(Clone, Default)]
struct AnimationTriggerRecord {
    animations: Vec<(i32, v8::Global<v8::Object>)>,
}

#[derive(Default)]
pub(crate) struct AnimationTriggerStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, AnimationTriggerRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(AnimationTriggerStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "AnimationTrigger", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<AnimationTriggerStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "AnimationTrigger",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_method(scope, prototype, "addAnimation", 2, add_animation)?;
    crate::webidl::define_method(scope, prototype, "getAnimations", 0, get_animations)?;
    crate::webidl::define_method(scope, prototype, "removeAnimation", 1, remove_animation)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<AnimationTriggerStore>()
        .ok_or_else(|| "AnimationTrigger state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

#[allow(dead_code)]
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let trigger = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, trigger, prototype.into()) != Some(true) {
        return Err("cannot create AnimationTrigger".to_owned());
    }
    attach(scope, trigger);
    Ok(trigger)
}

pub(crate) fn attach(scope: &mut v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) {
    if let Some(store) = scope.get_slot_mut::<AnimationTriggerStore>() {
        store.records.insert(
            object.get_identity_hash().get(),
            AnimationTriggerRecord::default(),
        );
    }
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'AnimationTrigger': Illegal constructor",
    );
}

fn add_animation(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'addAnimation' on 'AnimationTrigger': 1 argument required, but only 0 present.",
        );
        return;
    }
    let Ok(animation) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        throw_animation_type(scope, "addAnimation");
        return;
    };
    if !super::animation::is_instance(scope, animation) {
        throw_animation_type(scope, "addAnimation");
        return;
    }
    let identity = animation.get_identity_hash().get();
    let animation = v8::Global::new(scope, animation);
    let Some(record) = scope
        .get_slot_mut::<AnimationTriggerStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if !record.animations.iter().any(|entry| entry.0 == identity) {
        record.animations.push((identity, animation));
    }
}

fn get_animations(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(animations) = scope
        .get_slot::<AnimationTriggerStore>()
        .and_then(|store| {
            store
                .records
                .get(&arguments.this().get_identity_hash().get())
        })
        .map(|record| record.animations.clone())
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let values = v8::Array::new(scope, animations.len() as i32);
    for (index, animation) in animations.iter().enumerate() {
        let animation = v8::Local::new(scope, &animation.1);
        let _ = values.set_index(scope, index as u32, animation.into());
    }
    result.set(values.into());
}

fn remove_animation(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'removeAnimation' on 'AnimationTrigger': 1 argument required, but only 0 present.",
        );
        return;
    }
    let Ok(animation) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        throw_animation_type(scope, "removeAnimation");
        return;
    };
    if !super::animation::is_instance(scope, animation) {
        throw_animation_type(scope, "removeAnimation");
        return;
    }
    let Some(record) = scope
        .get_slot_mut::<AnimationTriggerStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let identity = animation.get_identity_hash().get();
    record.animations.retain(|entry| entry.0 != identity);
}

fn throw_animation_type(scope: &mut v8::PinScope<'_, '_>, method: &str) {
    crate::webidl::throw_type_error(
        scope,
        &format!(
            "Failed to execute '{method}' on 'AnimationTrigger': parameter 1 is not of type 'Animation'."
        ),
    );
}
