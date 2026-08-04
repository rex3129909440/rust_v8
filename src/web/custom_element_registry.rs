use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub(crate) struct CustomElementRegistryStore {
    constructors: HashMap<i32, v8::Global<v8::Function>>,
    instances: HashSet<i32>,
    definitions: HashMap<i32, HashMap<String, v8::Global<v8::Function>>>,
    waiters: HashMap<(i32, String), Vec<v8::Global<v8::PromiseResolver>>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CustomElementRegistryStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "CustomElementRegistry", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<CustomElementRegistryStore>()
        .and_then(|store| store.constructors.get(&crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "CustomElementRegistry",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_method(scope, prototype, "define", 2, define)?;
    crate::webidl::define_method(scope, prototype, "get", 1, get)?;
    crate::webidl::define_method(scope, prototype, "getName", 1, get_name)?;
    crate::webidl::define_method(scope, prototype, "upgrade", 1, upgrade)?;
    crate::webidl::define_method(scope, prototype, "whenDefined", 1, when_defined)?;
    crate::webidl::define_method(scope, prototype, "initialize", 1, initialize)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<CustomElementRegistryStore>()
        .ok_or_else(|| "CustomElementRegistry state was not prepared".to_owned())?
        .constructors
        .insert(realm_id, stored);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create CustomElementRegistry".to_owned());
    }
    attach(scope, object);
    Ok(object)
}

fn attach(scope: &mut v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) {
    let identity = object.get_identity_hash().get();
    if let Some(store) = scope.get_slot_mut::<CustomElementRegistryStore>() {
        store.instances.insert(identity);
        store.definitions.entry(identity).or_default();
    }
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(scope, "CustomElementRegistry requires new");
        return;
    }
    attach(scope, arguments.this());
    result.set(arguments.this().into());
}

fn instance_id(scope: &mut v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> Option<i32> {
    let identity = object.get_identity_hash().get();
    if scope
        .get_slot::<CustomElementRegistryStore>()
        .is_some_and(|store| store.instances.contains(&identity))
    {
        Some(identity)
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        None
    }
}

fn define(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(identity) = instance_id(scope, arguments.this()) else {
        return;
    };
    let name = crate::webidl::value_to_string(scope, arguments.get(0));
    let Ok(constructor) = v8::Local::<v8::Function>::try_from(arguments.get(1)) else {
        crate::webidl::throw_type_error(scope, "Custom element constructor must be callable");
        return;
    };
    let duplicate = scope
        .get_slot::<CustomElementRegistryStore>()
        .and_then(|store| store.definitions.get(&identity))
        .is_some_and(|definitions| definitions.contains_key(&name));
    if duplicate {
        crate::webidl::throw_type_error(scope, "The custom element name has already been used");
        return;
    }
    let constructor_global = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<CustomElementRegistryStore>()
        .expect("CustomElementRegistry state")
        .definitions
        .entry(identity)
        .or_default()
        .insert(name.clone(), constructor_global);
    let waiters = scope
        .get_slot_mut::<CustomElementRegistryStore>()
        .and_then(|store| store.waiters.remove(&(identity, name)))
        .unwrap_or_default();
    for waiter in waiters {
        let waiter = v8::Local::new(scope, &waiter);
        let _ = waiter.resolve(scope, constructor.into());
    }
}

fn get(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(identity) = instance_id(scope, arguments.this()) else {
        return;
    };
    let name = crate::webidl::value_to_string(scope, arguments.get(0));
    let constructor = scope
        .get_slot::<CustomElementRegistryStore>()
        .and_then(|store| store.definitions.get(&identity))
        .and_then(|definitions| definitions.get(&name))
        .cloned();
    match constructor {
        Some(constructor) => result.set(v8::Local::new(scope, &constructor).into()),
        None => result.set(v8::undefined(scope).into()),
    }
}

fn get_name(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(identity) = instance_id(scope, arguments.this()) else {
        return;
    };
    let Ok(wanted) = v8::Local::<v8::Function>::try_from(arguments.get(0)) else {
        result.set(v8::null(scope).into());
        return;
    };
    let found = scope
        .get_slot::<CustomElementRegistryStore>()
        .and_then(|store| store.definitions.get(&identity))
        .and_then(|definitions| {
            definitions.iter().find_map(|(name, constructor)| {
                let constructor = v8::Local::new(scope, constructor);
                constructor
                    .strict_equals(wanted.into())
                    .then(|| name.clone())
            })
        });
    match found {
        Some(name) => {
            if let Some(name) = v8::String::new(scope, &name) {
                result.set(name.into());
            }
        }
        None => result.set(v8::null(scope).into()),
    }
}

fn upgrade(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if instance_id(scope, arguments.this()).is_none() {
        return;
    }
    if v8::Local::<v8::Object>::try_from(arguments.get(0)).is_err() {
        crate::webidl::throw_type_error(scope, "upgrade requires a Node");
    }
}

fn when_defined(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(identity) = instance_id(scope, arguments.this()) else {
        return;
    };
    let name = crate::webidl::value_to_string(scope, arguments.get(0));
    let existing = scope
        .get_slot::<CustomElementRegistryStore>()
        .and_then(|store| store.definitions.get(&identity))
        .and_then(|definitions| definitions.get(&name))
        .cloned();
    if let Some(existing) = existing {
        if let Ok(promise) =
            super::writable_stream::resolved_promise(scope, v8::Local::new(scope, &existing).into())
        {
            result.set(promise.into());
        }
        return;
    }
    let Some(resolver) = v8::PromiseResolver::new(scope) else {
        crate::webidl::throw_type_error(scope, "cannot create promise");
        return;
    };
    let promise = resolver.get_promise(scope);
    let resolver = v8::Global::new(scope, resolver);
    scope
        .get_slot_mut::<CustomElementRegistryStore>()
        .expect("CustomElementRegistry state")
        .waiters
        .entry((identity, name))
        .or_default()
        .push(resolver);
    result.set(promise.into());
}

fn initialize(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if instance_id(scope, arguments.this()).is_none() {
        return;
    }
    if !arguments.get(0).is_undefined()
        && v8::Local::<v8::Object>::try_from(arguments.get(0)).is_err()
    {
        crate::webidl::throw_type_error(scope, "initialize requires a Document");
    }
}
