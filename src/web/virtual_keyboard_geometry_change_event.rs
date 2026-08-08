#[derive(Default)]
pub(crate) struct VirtualKeyboardGeometryChangeEventStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(VirtualKeyboardGeometryChangeEventStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(
        scope,
        "VirtualKeyboardGeometryChangeEvent",
        constructor.into(),
    )
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<VirtualKeyboardGeometryChangeEventStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "VirtualKeyboardGeometryChangeEvent",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let event = super::event::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, event)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<VirtualKeyboardGeometryChangeEventStore>()
        .ok_or_else(|| "VirtualKeyboardGeometryChangeEvent state was not prepared".to_owned())?
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
            "Failed to construct 'VirtualKeyboardGeometryChangeEvent': use new",
        );
        return;
    }
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(scope, "1 argument required");
        return;
    }
    let event_type = crate::webidl::value_to_string(scope, arguments.get(0));
    let init = v8::Local::<v8::Object>::try_from(arguments.get(1)).ok();
    let bubbles = init.is_some_and(|init| super::event::boolean_property(scope, init, "bubbles"));
    let cancelable =
        init.is_some_and(|init| super::event::boolean_property(scope, init, "cancelable"));
    let composed = init.is_some_and(|init| super::event::boolean_property(scope, init, "composed"));
    let object = arguments.this();
    super::event::attach(scope, object, event_type, bubbles, cancelable, composed);
    result.set(object.into());
}
