use std::collections::HashMap;

const TYPE_NAVIGATE: i32 = 0;
const TYPE_RELOAD: i32 = 1;
const TYPE_BACK_FORWARD: i32 = 2;
const TYPE_RESERVED: i32 = 255;

#[derive(Default)]
pub(crate) struct PerformanceNavigationStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, NavigationRecord>,
}

#[derive(Clone)]
struct NavigationRecord {
    navigation_type: i32,
    redirect_count: i32,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(PerformanceNavigationStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "PerformanceNavigation", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<PerformanceNavigationStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "PerformanceNavigation",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "type", get_type)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "redirectCount", get_redirect_count)?;
    define_constants(scope, prototype)?;
    crate::webidl::define_method(scope, prototype, "toJSON", 0, to_json)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    define_constants(scope, constructor.into())?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<PerformanceNavigationStore>()
        .ok_or_else(|| "PerformanceNavigation state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn define_constants(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_constant(scope, object, "TYPE_NAVIGATE", TYPE_NAVIGATE)?;
    crate::webidl::define_constant(scope, object, "TYPE_RELOAD", TYPE_RELOAD)?;
    crate::webidl::define_constant(scope, object, "TYPE_BACK_FORWARD", TYPE_BACK_FORWARD)?;
    crate::webidl::define_constant(scope, object, "TYPE_RESERVED", TYPE_RESERVED)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'PerformanceNavigation': Illegal constructor",
    );
}

#[allow(dead_code)]
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    navigation_type: i32,
    redirect_count: i32,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let navigation = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, navigation, prototype.into()) != Some(true) {
        return Err("cannot create PerformanceNavigation".to_owned());
    }
    scope
        .get_slot_mut::<PerformanceNavigationStore>()
        .ok_or_else(|| "PerformanceNavigation state was not prepared".to_owned())?
        .records
        .insert(
            navigation.get_identity_hash().get(),
            NavigationRecord {
                navigation_type,
                redirect_count,
            },
        );
    Ok(navigation)
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<NavigationRecord> {
    scope
        .get_slot::<PerformanceNavigationStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_type(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new(scope, record.navigation_type).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_redirect_count(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new(scope, record.redirect_count).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn define_number(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: i32,
) {
    if let Some(key) = v8::String::new(scope, name) {
        let _ =
            object.create_data_property(scope, key.into(), v8::Integer::new(scope, value).into());
    }
}

fn to_json(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let output = v8::Object::new(scope);
    define_number(scope, output, "type", record.navigation_type);
    define_number(scope, output, "redirectCount", record.redirect_count);
    result.set(output.into());
}
