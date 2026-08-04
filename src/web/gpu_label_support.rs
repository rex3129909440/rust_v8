use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct GpuLabelStore {
    labels: HashMap<i32, String>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(GpuLabelStore::default());
}

pub(crate) fn attach(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    label: String,
) {
    scope
        .get_slot_mut::<GpuLabelStore>()
        .expect("GPU label state")
        .labels
        .insert(object.get_identity_hash().get(), label);
}

pub(crate) fn get(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<String> {
    scope
        .get_slot::<GpuLabelStore>()?
        .labels
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn set(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    label: String,
) -> bool {
    let Some(current) = scope
        .get_slot_mut::<GpuLabelStore>()
        .and_then(|store| store.labels.get_mut(&object.get_identity_hash().get()))
    else {
        return false;
    };
    *current = label;
    true
}

pub(crate) fn label_from_descriptor(
    scope: &v8::PinScope<'_, '_>,
    descriptor: v8::Local<'_, v8::Value>,
) -> String {
    let Ok(object) = v8::Local::<v8::Object>::try_from(descriptor) else {
        return String::new();
    };
    let Some(key) = v8::String::new(scope, "label") else {
        return String::new();
    };
    object
        .get(scope, key.into())
        .filter(|value| !value.is_undefined())
        .map(|value| crate::webidl::value_to_string(scope, value))
        .unwrap_or_default()
}
