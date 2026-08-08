use std::collections::HashSet;

#[derive(Default)]
pub(crate) struct LaunchQueueStore {
    constructor: crate::webidl::RealmConstructor,
    native_objects: HashSet<i32>,
    consumer: Option<v8::Global<v8::Function>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(LaunchQueueStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "LaunchQueue", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<LaunchQueueStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "LaunchQueue",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_method(scope, prototype, "setConsumer", 1, set_consumer)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<LaunchQueueStore>()
        .ok_or_else(|| "LaunchQueue state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let queue = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, queue, prototype.into()) != Some(true) {
        return Err("cannot create LaunchQueue".to_owned());
    }
    scope
        .get_slot_mut::<LaunchQueueStore>()
        .ok_or_else(|| "LaunchQueue state was not prepared".to_owned())?
        .native_objects
        .insert(queue.get_identity_hash().get());
    Ok(queue)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'LaunchQueue': Illegal constructor",
    );
}

fn set_consumer(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let is_native = scope.get_slot::<LaunchQueueStore>().is_some_and(|store| {
        store
            .native_objects
            .contains(&arguments.this().get_identity_hash().get())
    });
    if !is_native {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'setConsumer' on 'LaunchQueue': 1 argument required, but only 0 present.",
        );
        return;
    }
    let Ok(consumer) = v8::Local::<v8::Function>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'setConsumer' on 'LaunchQueue': parameter 1 is not of type 'Function'.",
        );
        return;
    };
    let consumer = v8::Global::new(scope, consumer);
    if let Some(store) = scope.get_slot_mut::<LaunchQueueStore>() {
        store.consumer = Some(consumer);
    }
}

#[allow(dead_code)]
pub(crate) fn deliver(
    scope: &mut v8::PinScope<'_, '_>,
    launch_params: v8::Local<'_, v8::Object>,
) -> bool {
    let consumer = scope
        .get_slot::<LaunchQueueStore>()
        .and_then(|store| store.consumer.clone());
    let Some(consumer) = consumer else {
        return false;
    };
    let consumer = v8::Local::new(scope, &consumer);
    let undefined = v8::undefined(scope);
    consumer
        .call(scope, undefined.into(), &[launch_params.into()])
        .is_some()
}
