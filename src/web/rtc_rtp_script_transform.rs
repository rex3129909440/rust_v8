use std::collections::HashMap;

#[derive(Clone)]
struct RtcRtpScriptTransformRecord {
    worker: v8::Global<v8::Object>,
    options: v8::Global<v8::Value>,
}

#[derive(Default)]
pub(crate) struct RtcRtpScriptTransformStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, RtcRtpScriptTransformRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(RtcRtpScriptTransformStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "RTCRtpScriptTransform", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<RtcRtpScriptTransformStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "RTCRtpScriptTransform",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<RtcRtpScriptTransformStore>()
        .ok_or_else(|| "RTCRtpScriptTransform state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'RTCRtpScriptTransform': Please use the 'new' operator.",
        );
        return;
    }
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'RTCRtpScriptTransform': 1 argument required, but only 0 present.",
        );
        return;
    }
    let Ok(worker) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'RTCRtpScriptTransform': parameter 1 is not of type 'Worker'.",
        );
        return;
    };
    if !super::structured_clone::inherits_platform_interface(scope, worker, "Worker") {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'RTCRtpScriptTransform': parameter 1 is not of type 'Worker'.",
        );
        return;
    }
    let record = RtcRtpScriptTransformRecord {
        worker: v8::Global::new(scope, worker),
        options: v8::Global::new(scope, arguments.get(1)),
    };
    scope
        .get_slot_mut::<RtcRtpScriptTransformStore>()
        .expect("RTCRtpScriptTransform state")
        .records
        .insert(arguments.this().get_identity_hash().get(), record);
    if let Some(realm_id) = super::worker::realm_id_for(scope, worker) {
        let _ =
            super::worker_global_scope::dispatch_rtc_transform(scope, realm_id, arguments.get(1));
    }
    result.set(arguments.this().into());
}

pub(crate) fn is_instance(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> bool {
    scope
        .get_slot::<RtcRtpScriptTransformStore>()
        .is_some_and(|store| {
            store
                .records
                .get(&object.get_identity_hash().get())
                .is_some_and(|record| {
                    let _ = &record.worker;
                    let _ = &record.options;
                    true
                })
        })
}
