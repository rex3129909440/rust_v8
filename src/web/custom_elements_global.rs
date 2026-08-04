use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct CustomElementsGlobalStore {
    values: HashMap<i32, v8::Global<v8::Object>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CustomElementsGlobalStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let document = super::document_global::value(scope);
    install_with_document(scope, document)
}

pub(crate) fn install_for_document(
    scope: &mut v8::PinScope<'_, '_>,
    document: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    install_with_document(scope, Some(document))
}

fn install_with_document(
    scope: &mut v8::PinScope<'_, '_>,
    document: Option<v8::Local<'_, v8::Object>>,
) -> Result<(), String> {
    let registry = super::custom_element_registry::create(scope)?;
    if let Some(document) = document {
        super::document::set_object_value(scope, document, "customElementRegistry", registry);
    }
    let realm_id = crate::webidl::realm_id(scope);
    let stored = v8::Global::new(scope, registry);
    scope
        .get_slot_mut::<CustomElementsGlobalStore>()
        .ok_or_else(|| "customElements global state was not prepared".to_owned())?
        .values
        .insert(realm_id, stored);
    let getter = crate::webidl::create_function(
        scope,
        "get customElements",
        0,
        v8::ConstructorBehavior::Throw,
        get_custom_elements,
    )?;
    let undefined = v8::undefined(scope);
    let mut descriptor = v8::PropertyDescriptor::new_from_get_set(getter.into(), undefined.into());
    descriptor.set_enumerable(true);
    descriptor.set_configurable(true);
    let key = crate::webidl::string(scope, "customElements")?;
    let global = scope.get_current_context().global(scope);
    if global.define_property(scope, key.into(), &descriptor) == Some(true) {
        Ok(())
    } else {
        Err("cannot define window.customElements".to_owned())
    }
}

fn get_custom_elements(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(value) = scope
        .get_slot::<CustomElementsGlobalStore>()
        .and_then(|store| store.values.get(&crate::webidl::realm_id(scope)))
        .cloned()
    {
        result.set(v8::Local::new(scope, &value).into());
    }
}
