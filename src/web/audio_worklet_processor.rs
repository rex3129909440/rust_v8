use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct AudioWorkletProcessorStore {
    pending_port: Option<v8::Global<v8::Object>>,
    records: HashMap<i32, v8::Global<v8::Object>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(AudioWorkletProcessorStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = crate::webidl::create_function(
        scope,
        "AudioWorkletProcessor",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "port", get_port)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::define_global(scope, "AudioWorkletProcessor", constructor.into())
}

pub(crate) fn set_pending_port(scope: &mut v8::PinScope<'_, '_>, port: v8::Local<'_, v8::Object>) {
    let port = v8::Global::new(scope, port);
    if let Some(store) = scope.get_slot_mut::<AudioWorkletProcessorStore>() {
        store.pending_port = Some(port);
    }
}

pub(crate) fn clear_pending_port(scope: &mut v8::PinScope<'_, '_>) {
    if let Some(store) = scope.get_slot_mut::<AudioWorkletProcessorStore>() {
        store.pending_port = None;
    }
}

pub(crate) fn is_instance(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> bool {
    scope
        .get_slot::<AudioWorkletProcessorStore>()
        .is_some_and(|store| {
            store
                .records
                .contains_key(&object.get_identity_hash().get())
        })
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'AudioWorkletProcessor': use the 'new' operator",
        );
        return;
    }
    let Some(port) = scope
        .get_slot_mut::<AudioWorkletProcessorStore>()
        .and_then(|store| store.pending_port.take())
    else {
        crate::webidl::throw_type_error(
            scope,
            "AudioWorkletProcessor can only be constructed by AudioWorkletNode",
        );
        return;
    };
    scope
        .get_slot_mut::<AudioWorkletProcessorStore>()
        .expect("AudioWorkletProcessor state")
        .records
        .insert(arguments.this().get_identity_hash().get(), port);
    result.set(arguments.this().into());
}

fn get_port(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let port = scope
        .get_slot::<AudioWorkletProcessorStore>()
        .and_then(|store| {
            store
                .records
                .get(&arguments.this().get_identity_hash().get())
        })
        .cloned();
    if let Some(port) = port {
        result.set(v8::Local::new(scope, &port).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
