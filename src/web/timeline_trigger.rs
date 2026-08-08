use std::collections::HashMap;

#[derive(Clone)]
struct TimelineTriggerRecord {
    ranges: v8::Global<v8::Object>,
}

#[derive(Default)]
pub(crate) struct TimelineTriggerStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, TimelineTriggerRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(TimelineTriggerStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "TimelineTrigger", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<TimelineTriggerStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "TimelineTrigger",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "ranges", get_ranges)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::animation_trigger::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<TimelineTriggerStore>()
        .ok_or_else(|| "TimelineTrigger state was not prepared".to_owned())?
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
            "Failed to construct 'TimelineTrigger': Please use the 'new' operator.",
        );
        return;
    }
    let mut ranges = Vec::new();
    if !arguments.get(0).is_undefined() {
        let Ok(init_ranges) = v8::Local::<v8::Array>::try_from(arguments.get(0)) else {
            crate::webidl::throw_type_error(
                scope,
                "Failed to construct 'TimelineTrigger': The object must have a callable @@iterator property.",
            );
            return;
        };
        for index in 0..init_ranges.length() {
            let Some(value) = init_ranges.get_index(scope, index) else {
                continue;
            };
            let Ok(init) = v8::Local::<v8::Object>::try_from(value) else {
                crate::webidl::throw_type_error(
                    scope,
                    "Failed to construct 'TimelineTrigger': range entry must be an object.",
                );
                return;
            };
            let Ok(range) = super::timeline_trigger_range::create(scope, init) else {
                return;
            };
            ranges.push(range);
        }
    }
    let Ok(list) = super::timeline_trigger_range_list::create(scope, ranges) else {
        return;
    };
    super::animation_trigger::attach(scope, arguments.this());
    let ranges_global = v8::Global::new(scope, list);
    scope
        .get_slot_mut::<TimelineTriggerStore>()
        .expect("TimelineTrigger state")
        .records
        .insert(
            arguments.this().get_identity_hash().get(),
            TimelineTriggerRecord {
                ranges: ranges_global,
            },
        );
    result.set(arguments.this().into());
}

fn get_ranges(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = scope
        .get_slot::<TimelineTriggerStore>()
        .and_then(|store| {
            store
                .records
                .get(&arguments.this().get_identity_hash().get())
        })
        .cloned()
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    result.set(v8::Local::new(scope, &record.ranges).into());
}
