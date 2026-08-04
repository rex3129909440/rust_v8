use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct DocumentPictureInPictureGlobalStore {
    values: HashMap<i32, v8::Global<v8::Object>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(DocumentPictureInPictureGlobalStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let value = super::document_picture_in_picture::create(scope)?;
    let stored = v8::Global::new(scope, value);
    let realm_id = crate::webidl::realm_id(scope);
    scope
        .get_slot_mut::<DocumentPictureInPictureGlobalStore>()
        .ok_or_else(|| "documentPictureInPicture global state was not prepared".to_owned())?
        .values
        .insert(realm_id, stored);
    let getter = crate::webidl::create_function(
        scope,
        "get documentPictureInPicture",
        0,
        v8::ConstructorBehavior::Throw,
        get_value,
    )?;
    let undefined = v8::undefined(scope);
    let mut descriptor = v8::PropertyDescriptor::new_from_get_set(getter.into(), undefined.into());
    descriptor.set_enumerable(true);
    descriptor.set_configurable(true);
    let global = scope.get_current_context().global(scope);
    let key = crate::webidl::string(scope, "documentPictureInPicture")?;
    if global.define_property(scope, key.into(), &descriptor) == Some(true) {
        Ok(())
    } else {
        Err("cannot define window.documentPictureInPicture".to_owned())
    }
}

fn get_value(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(value) = scope
        .get_slot::<DocumentPictureInPictureGlobalStore>()
        .and_then(|store| store.values.get(&crate::webidl::realm_id(scope)))
        .cloned()
    {
        result.set(v8::Local::new(scope, &value).into());
    }
}
