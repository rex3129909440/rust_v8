use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct InputDeviceCapabilitiesStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, bool>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(InputDeviceCapabilitiesStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "InputDeviceCapabilities", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<InputDeviceCapabilitiesStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "InputDeviceCapabilities",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "firesTouchEvents",
        get_fires_touch_events,
    )?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<InputDeviceCapabilitiesStore>()
        .ok_or_else(|| "InputDeviceCapabilities state was not prepared".to_owned())?
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
            "Failed to construct 'InputDeviceCapabilities': Please use the 'new' operator",
        );
        return;
    }
    let fires_touch_events = v8::Local::<v8::Object>::try_from(arguments.get(0))
        .ok()
        .is_some_and(|options| {
            let Some(key) = v8::String::new(scope, "firesTouchEvents") else {
                return false;
            };
            options
                .get(scope, key.into())
                .is_some_and(|value| value.boolean_value(scope))
        });
    scope
        .get_slot_mut::<InputDeviceCapabilitiesStore>()
        .expect("InputDeviceCapabilities state")
        .records
        .insert(
            arguments.this().get_identity_hash().get(),
            fires_touch_events,
        );
    result.set(arguments.this().into());
}

fn get_fires_touch_events(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let value = scope
        .get_slot::<InputDeviceCapabilitiesStore>()
        .and_then(|store| {
            store
                .records
                .get(&arguments.this().get_identity_hash().get())
        })
        .copied();
    if let Some(value) = value {
        result.set(v8::Boolean::new(scope, value).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
