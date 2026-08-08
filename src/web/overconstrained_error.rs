use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct OverconstrainedErrorStore {
    constructor: crate::webidl::RealmConstructor,
    constraints: HashMap<i32, String>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(OverconstrainedErrorStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "OverconstrainedError", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<OverconstrainedErrorStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "OverconstrainedError",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "constraint", get_constraint)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::dom_exception::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<OverconstrainedErrorStore>()
        .ok_or_else(|| "OverconstrainedError state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'OverconstrainedError': 1 argument required",
        );
        return;
    }
    let constraint = crate::webidl::value_to_string(scope, arguments.get(0));
    let message = if arguments.get(1).is_undefined() {
        String::new()
    } else {
        crate::webidl::value_to_string(scope, arguments.get(1))
    };
    super::dom_exception::attach(
        scope,
        arguments.this(),
        "OverconstrainedError".to_owned(),
        message,
        0,
    );
    scope
        .get_slot_mut::<OverconstrainedErrorStore>()
        .expect("OverconstrainedError state")
        .constraints
        .insert(arguments.this().get_identity_hash().get(), constraint);
    result.set(arguments.this().into());
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    constraint: String,
    message: String,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create OverconstrainedError".to_owned());
    }
    super::dom_exception::attach(scope, object, "OverconstrainedError".to_owned(), message, 0);
    scope
        .get_slot_mut::<OverconstrainedErrorStore>()
        .ok_or_else(|| "OverconstrainedError state was not prepared".to_owned())?
        .constraints
        .insert(object.get_identity_hash().get(), constraint);
    Ok(object)
}

fn get_constraint(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(constraint) = scope
        .get_slot::<OverconstrainedErrorStore>()
        .and_then(|store| {
            store
                .constraints
                .get(&arguments.this().get_identity_hash().get())
        })
    {
        if let Some(value) = v8::String::new(scope, constraint) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
