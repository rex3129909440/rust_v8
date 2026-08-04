use std::collections::HashMap;

#[derive(Clone)]
struct BatteryManagerRecord {
    charging: bool,
    charging_time: f64,
    discharging_time: f64,
    level: f64,
    handlers: HashMap<&'static str, v8::Global<v8::Value>>,
}

#[derive(Default)]
pub(crate) struct BatteryManagerStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, BatteryManagerRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(BatteryManagerStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "BatteryManager", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<BatteryManagerStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let parent = super::event_target::ensure_constructor(scope)?;
    let constructor = crate::webidl::create_function(
        scope,
        "BatteryManager",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "charging", get_charging)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "chargingTime", get_charging_time)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "dischargingTime",
        get_discharging_time,
    )?;
    crate::webidl::define_readonly_accessor(scope, prototype, "level", get_level)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "onchargingchange",
        get_onchargingchange,
        set_onchargingchange,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "onchargingtimechange",
        get_onchargingtimechange,
        set_onchargingtimechange,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "ondischargingtimechange",
        get_ondischargingtimechange,
        set_ondischargingtimechange,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "onlevelchange",
        get_onlevelchange,
        set_onlevelchange,
    )?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<BatteryManagerStore>()
        .ok_or_else(|| "BatteryManager state was not prepared".to_owned())?
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
        "Failed to construct 'BatteryManager': Illegal constructor",
    );
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create BatteryManager".to_owned());
    }
    super::event_target::attach(scope, object);
    let configured = crate::fingerprint::edge(scope).battery.clone();
    scope
        .get_slot_mut::<BatteryManagerStore>()
        .ok_or_else(|| "BatteryManager state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            BatteryManagerRecord {
                charging: configured.charging,
                charging_time: configured.charging_time,
                discharging_time: configured.discharging_time,
                level: configured.level,
                handlers: HashMap::new(),
            },
        );
    Ok(object)
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<BatteryManagerRecord> {
    scope
        .get_slot::<BatteryManagerStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_charging(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, record.charging).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
fn get_charging_time(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, record.charging_time).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
fn get_discharging_time(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, record.discharging_time).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
fn get_level(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, record.level).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn handler_get(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &'static str,
    result: v8::ReturnValue<'_>,
) {
    let handler = record(scope, object).and_then(|record| record.handlers.get(name).cloned());
    super::window_event_handler_support::return_handler(scope, handler, result);
}

fn handler_set(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    value: v8::Local<'_, v8::Value>,
    name: &'static str,
) {
    let handler = super::window_event_handler_support::handler_value(scope, value);
    let Some(record) = scope
        .get_slot_mut::<BatteryManagerStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    match handler {
        Some(handler) => {
            record.handlers.insert(name, handler);
        }
        None => {
            record.handlers.remove(name);
        }
    }
}

macro_rules! battery_handler {
    ($get:ident, $set:ident, $name:literal) => {
        fn $get(
            scope: &mut v8::PinScope<'_, '_>,
            arguments: v8::FunctionCallbackArguments<'_>,
            result: v8::ReturnValue<'_>,
        ) {
            handler_get(scope, arguments.this(), $name, result);
        }
        fn $set(
            scope: &mut v8::PinScope<'_, '_>,
            arguments: v8::FunctionCallbackArguments<'_>,
            _: v8::ReturnValue<'_>,
        ) {
            handler_set(scope, arguments.this(), arguments.get(0), $name);
        }
    };
}

battery_handler!(get_onchargingchange, set_onchargingchange, "chargingchange");
battery_handler!(
    get_onchargingtimechange,
    set_onchargingtimechange,
    "chargingtimechange"
);
battery_handler!(
    get_ondischargingtimechange,
    set_ondischargingtimechange,
    "dischargingtimechange"
);
battery_handler!(get_onlevelchange, set_onlevelchange, "levelchange");
