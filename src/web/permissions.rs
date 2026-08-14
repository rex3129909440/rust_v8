use std::collections::HashSet;

#[derive(Default)]
pub(crate) struct PermissionsStore {
    constructor: crate::webidl::RealmConstructor,
    identities: HashSet<i32>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(PermissionsStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "Permissions", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<PermissionsStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "Permissions",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_method(scope, prototype, "query", 1, query)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<PermissionsStore>()
        .ok_or_else(|| "Permissions state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'Permissions': Illegal constructor",
    );
}

#[allow(dead_code)]
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let permissions = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, permissions, prototype.into()) != Some(true) {
        return Err("cannot create Permissions".to_owned());
    }
    scope
        .get_slot_mut::<PermissionsStore>()
        .ok_or_else(|| "Permissions state was not prepared".to_owned())?
        .identities
        .insert(permissions.get_identity_hash().get());
    Ok(permissions)
}

fn is_instance(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> bool {
    scope
        .get_slot::<PermissionsStore>()
        .is_some_and(|store| store.identities.contains(&object.get_identity_hash().get()))
}

fn query(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !is_instance(scope, arguments.this()) {
        crate::webidl::reject_illegal_invocation_promise(scope, "Permissions", "query", result);
        return;
    }
    let Ok(descriptor) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "PermissionDescriptor must be an object");
        return;
    };
    let Some(name_key) = v8::String::new(scope, "name") else {
        return;
    };
    let Some(name_value) = descriptor.get(scope, name_key.into()) else {
        crate::webidl::throw_type_error(scope, "Required member 'name' is undefined");
        return;
    };
    if name_value.is_undefined() {
        crate::webidl::throw_type_error(scope, "Required member 'name' is undefined");
        return;
    }
    let name = crate::webidl::value_to_string(scope, name_value);
    if !valid_permission_name(&name) {
        crate::webidl::throw_type_error(scope, "The provided permission name is not valid");
        return;
    }
    let state = crate::fingerprint::edge(scope)
        .permissions
        .state(&name)
        .expect("validated permission name")
        .to_owned();
    let Some(resolver) = v8::PromiseResolver::new(scope) else {
        return;
    };
    let promise = resolver.get_promise(scope);
    let rejection_message = match state.as_str() {
        "unsupported" => Some(
            "Failed to execute 'query' on 'Permissions': The Speaker Selection API is not enabled.",
        ),
        "invalid-origin" => {
            Some("Failed to execute 'query' on 'Permissions': The requested origin is invalid.")
        }
        _ => None,
    };
    if let Some(message) = rejection_message {
        if let Some(message) = v8::String::new(scope, message) {
            let error = v8::Exception::type_error(scope, message);
            let _ = resolver.reject(scope, error);
            result.set(promise.into());
        }
        return;
    }
    let status = match super::permission_status::create(scope, name, state) {
        Ok(status) => status,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    let _ = resolver.resolve(scope, status.into());
    result.set(promise.into());
}

fn valid_permission_name(name: &str) -> bool {
    matches!(
        name,
        "accelerometer"
            | "background-sync"
            | "camera"
            | "clipboard-read"
            | "clipboard-write"
            | "geolocation"
            | "gyroscope"
            | "local-fonts"
            | "magnetometer"
            | "microphone"
            | "midi"
            | "notifications"
            | "payment-handler"
            | "persistent-storage"
            | "speaker-selection"
            | "storage-access"
            | "top-level-storage-access"
            | "window-management"
    )
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<PermissionsStore>() {
        store.constructor.remove(realm_id);
    }
}
