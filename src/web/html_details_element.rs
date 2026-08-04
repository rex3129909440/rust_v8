use std::collections::HashMap;

#[derive(Clone)]
pub(crate) struct DetailsRecord {
    pub(crate) open: bool,
    pub(crate) name: String,
    pub(crate) toggle_pending: bool,
    pub(crate) object: v8::Global<v8::Object>,
}

#[derive(Default)]
pub(crate) struct HtmlDetailsElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, DetailsRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(HtmlDetailsElementStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "HTMLDetailsElement", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(constructor) = scope
        .get_slot::<HtmlDetailsElementStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let parent = super::html_element::ensure_constructor(scope)?;
    let constructor = crate::webidl::create_function(
        scope,
        "HTMLDetailsElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::html_details_element_open_property::define(scope, prototype)?;
    super::html_details_element_name_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<HtmlDetailsElementStore>()
        .ok_or_else(|| "HTMLDetailsElement state was not prepared".to_owned())?
        .constructor
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
        return Err("cannot create HTMLDetailsElement".to_owned());
    }
    super::html_element::attach(scope, object, "DETAILS");
    let stored_object = v8::Global::new(scope, object);
    scope
        .get_slot_mut::<HtmlDetailsElementStore>()
        .ok_or_else(|| "HTMLDetailsElement state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            DetailsRecord {
                open: false,
                name: String::new(),
                toggle_pending: false,
                object: stored_object,
            },
        );
    Ok(object)
}

pub(crate) fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor");
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<DetailsRecord> {
    scope
        .get_slot::<HtmlDetailsElementStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn get_open(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, record.open).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_open(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let next = arguments.get(0).boolean_value(scope);
    let identity = arguments.this().get_identity_hash().get();
    let should_queue = scope
        .get_slot_mut::<HtmlDetailsElementStore>()
        .and_then(|store| store.records.get_mut(&identity))
        .map(|record| {
            if record.open == next {
                return false;
            }
            record.open = next;
            if record.toggle_pending {
                false
            } else {
                record.toggle_pending = true;
                true
            }
        });
    let Some(should_queue) = should_queue else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if next {
        super::element::set_attribute_full(
            scope,
            arguments.this(),
            "open".to_owned(),
            String::new(),
            None,
        );
    } else {
        super::element::remove_attribute_full(scope, arguments.this(), None, "open");
    }
    if should_queue {
        let data = v8::Integer::new(scope, identity);
        if let Some(callback) = v8::Function::builder(deliver_toggle)
            .data(data.into())
            .length(0)
            .constructor_behavior(v8::ConstructorBehavior::Throw)
            .build(scope)
        {
            scope.enqueue_microtask(callback);
        }
    }
}

pub(crate) fn deliver_toggle(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(identity) = arguments.data().int32_value(scope) else {
        return;
    };
    let target = scope
        .get_slot_mut::<HtmlDetailsElementStore>()
        .and_then(|store| store.records.get_mut(&identity))
        .map(|record| {
            record.toggle_pending = false;
            record.object.clone()
        });
    if let Some(target) = target {
        let target = v8::Local::new(scope, &target);
        let event = super::event_target::create_event(scope, "toggle");
        super::event_target::dispatch(scope, target, event);
    }
}

pub(crate) fn get_name(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, &record.name) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_name(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    if let Some(record) = scope
        .get_slot_mut::<HtmlDetailsElementStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.name = value;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
