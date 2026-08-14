use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct FormDataEventStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, v8::Global<v8::Object>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(FormDataEventStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "FormDataEvent", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<FormDataEventStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "FormDataEvent",
        2,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let parent = super::event::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::form_data_event_form_data_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<FormDataEventStore>()
        .ok_or_else(|| "FormDataEvent state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 2 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'FormDataEvent': 2 arguments required, but only 1 present.",
        );
        return;
    }
    let Some(event_type) = crate::webidl::dom_string(scope, arguments.get(0)) else {
        return;
    };
    let Ok(init) = v8::Local::<v8::Object>::try_from(arguments.get(1)) else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'FormDataEvent': The provided value is not of type 'FormDataEventInit'.",
        );
        return;
    };
    let Some(key) = v8::String::new(scope, "formData") else {
        return;
    };
    let Some(value) = init.get(scope, key.into()) else {
        return;
    };
    let Ok(form_data) = v8::Local::<v8::Object>::try_from(value) else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'FormDataEvent': Failed to read the 'formData' property from 'FormDataEventInit': Required member is undefined.",
        );
        return;
    };
    if !super::form_data::is_form_data(scope, form_data) {
        crate::webidl::throw_type_error(scope, "formData is not of type FormData");
        return;
    }
    let bubbles = super::event::boolean_property(scope, init, "bubbles");
    let cancelable = super::event::boolean_property(scope, init, "cancelable");
    let composed = super::event::boolean_property(scope, init, "composed");
    super::event::attach(
        scope,
        arguments.this(),
        event_type,
        bubbles,
        cancelable,
        composed,
    );
    let form_data = v8::Global::new(scope, form_data);
    scope
        .get_slot_mut::<FormDataEventStore>()
        .expect("FormDataEvent state")
        .records
        .insert(arguments.this().get_identity_hash().get(), form_data);
    result.set(arguments.this().into());
}

pub(crate) fn get_form_data(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(value) = scope
        .get_slot::<FormDataEventStore>()
        .and_then(|store| {
            store
                .records
                .get(&arguments.this().get_identity_hash().get())
        })
        .cloned()
    {
        result.set(v8::Local::new(scope, &value).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
