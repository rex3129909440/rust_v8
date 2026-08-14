use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct UiEventStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, UiEventRecord>,
}

#[derive(Clone)]
pub(crate) struct UiEventRecord {
    pub(crate) view: Option<v8::Global<v8::Value>>,
    pub(crate) detail: i32,
    pub(crate) source_capabilities: Option<v8::Global<v8::Value>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(UiEventStore::default());
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<UiEventStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let parent = super::event::ensure_constructor(scope)?;
    let constructor = crate::webidl::create_function(
        scope,
        "UIEvent",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::ui_event_view_property::define(scope, prototype)?;
    super::ui_event_detail_property::define(scope, prototype)?;
    super::ui_event_source_capabilities_property::define(scope, prototype)?;
    super::ui_event_which_property::define(scope, prototype)?;
    super::ui_event_init_ui_event::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    super::ui_event_pseudo_target_property::define(scope, prototype)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<UiEventStore>()
        .ok_or_else(|| "UIEvent state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let event_type = crate::webidl::string(scope, "")?;
    constructor
        .new_instance(scope, &[event_type.into()])
        .ok_or_else(|| "cannot create UIEvent".to_owned())
}

#[allow(dead_code)]
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "UIEvent", constructor.into())
}

pub(crate) fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'UIEvent': Please use the 'new' operator",
        );
        return;
    }
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'UIEvent': 1 argument required",
        );
        return;
    }
    let event_type = crate::webidl::value_to_string(scope, arguments.get(0));
    let init = v8::Local::<v8::Object>::try_from(arguments.get(1)).ok();
    let bubbles = init.is_some_and(|init| super::event::boolean_property(scope, init, "bubbles"));
    let cancelable =
        init.is_some_and(|init| super::event::boolean_property(scope, init, "cancelable"));
    let composed = init.is_some_and(|init| super::event::boolean_property(scope, init, "composed"));
    let view = init.and_then(|init| value_property(scope, init, "view"));
    let detail = init
        .map(|init| super::event::number_property(scope, init, "detail", 0.0) as i32)
        .unwrap_or(0);
    let source_capabilities =
        init.and_then(|init| value_property(scope, init, "sourceCapabilities"));
    attach(
        scope,
        arguments.this(),
        event_type,
        bubbles,
        cancelable,
        composed,
        view,
        detail,
        source_capabilities,
    );
    result.set(arguments.this().into());
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn attach(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    event_type: String,
    bubbles: bool,
    cancelable: bool,
    composed: bool,
    view: Option<v8::Global<v8::Value>>,
    detail: i32,
    source_capabilities: Option<v8::Global<v8::Value>>,
) {
    super::event::attach(scope, object, event_type, bubbles, cancelable, composed);
    if let Some(store) = scope.get_slot_mut::<UiEventStore>() {
        store.records.insert(
            object.get_identity_hash().get(),
            UiEventRecord {
                view,
                detail,
                source_capabilities,
            },
        );
    }
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<UiEventRecord> {
    scope
        .get_slot::<UiEventStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn set_source_capabilities(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    capabilities: Option<v8::Local<'_, v8::Object>>,
) {
    let capabilities =
        capabilities.map(|value| v8::Global::new(scope, v8::Local::<v8::Value>::from(value)));
    if let Some(record) = scope
        .get_slot_mut::<UiEventStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        record.source_capabilities = capabilities;
    }
}

pub(crate) fn get_view(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(view) = record.view {
        result.set(v8::Local::new(scope, &view));
    } else {
        result.set(v8::null(scope).into());
    }
}

pub(crate) fn get_detail(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new(scope, record.detail).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_source_capabilities(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(value) = record.source_capabilities {
        result.set(v8::Local::new(scope, &value));
    } else {
        result.set(v8::null(scope).into());
    }
}

pub(crate) fn get_which(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(keyboard) = super::keyboard_event::record(scope, arguments.this()) {
        result.set(v8::Integer::new_from_unsigned(scope, keyboard.which).into());
    } else if record(scope, arguments.this()).is_some() {
        result.set(v8::Integer::new(scope, 0).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_pseudo_target(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_some() {
        result.set(v8::null(scope).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn init_ui_event(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let id = arguments.this().get_identity_hash().get();
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let event_type = crate::webidl::value_to_string(scope, arguments.get(0));
    let bubbles = arguments.get(1).boolean_value(scope);
    let cancelable = arguments.get(2).boolean_value(scope);
    let view = (!arguments.get(3).is_null() && !arguments.get(3).is_undefined())
        .then(|| v8::Global::new(scope, arguments.get(3)));
    let detail = arguments.get(4).int32_value(scope).unwrap_or(0);
    super::event::attach(
        scope,
        arguments.this(),
        event_type,
        bubbles,
        cancelable,
        false,
    );
    if let Some(record) = scope
        .get_slot_mut::<UiEventStore>()
        .and_then(|store| store.records.get_mut(&id))
    {
        record.view = view;
        record.detail = detail;
    }
}

pub(crate) fn value_property(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<v8::Global<v8::Value>> {
    let key = v8::String::new(scope, name)?;
    let value = object.get(scope, key.into())?;
    if value.is_null() || value.is_undefined() {
        None
    } else {
        Some(v8::Global::new(scope, value))
    }
}
