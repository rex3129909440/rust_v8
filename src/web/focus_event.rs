use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct FocusEventStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) related_targets: HashMap<i32, Option<v8::Global<v8::Object>>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(FocusEventStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "FocusEvent", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<FocusEventStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "FocusEvent",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let parent = super::ui_event::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::focus_event_related_target_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<FocusEventStore>()
        .ok_or_else(|| "FocusEvent state was not prepared".to_owned())?
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
        .ok_or_else(|| "cannot create FocusEvent".to_owned())
}

pub(crate) fn create_with_data<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    event_type: &str,
    bubbles: bool,
    composed: bool,
    related_target: Option<v8::Global<v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let event = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, event, prototype.into()) != Some(true) {
        return Err("cannot create FocusEvent".to_owned());
    }
    let view: v8::Local<v8::Value> = scope.get_current_context().global(scope).into();
    super::ui_event::attach(
        scope,
        event,
        event_type.to_owned(),
        bubbles,
        false,
        composed,
        Some(v8::Global::new(scope, view)),
        0,
        None,
    );
    scope
        .get_slot_mut::<FocusEventStore>()
        .ok_or_else(|| "FocusEvent state was not prepared".to_owned())?
        .related_targets
        .insert(event.get_identity_hash().get(), related_target);
    Ok(event)
}

pub(crate) fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'FocusEvent': 1 argument required",
        );
        return;
    }
    let event_type = crate::webidl::value_to_string(scope, arguments.get(0));
    let init = v8::Local::<v8::Object>::try_from(arguments.get(1)).ok();
    let bubbles = init.is_some_and(|value| super::event::boolean_property(scope, value, "bubbles"));
    let cancelable =
        init.is_some_and(|value| super::event::boolean_property(scope, value, "cancelable"));
    let composed =
        init.is_some_and(|value| super::event::boolean_property(scope, value, "composed"));
    let detail = init
        .map(|value| super::event::number_property(scope, value, "detail", 0.0) as i32)
        .unwrap_or(0);
    let view = init.and_then(|value| optional_value(scope, value, "view"));
    let source_capabilities =
        init.and_then(|value| optional_value(scope, value, "sourceCapabilities"));
    let related_target = init
        .and_then(|value| {
            let key = v8::String::new(scope, "relatedTarget")?;
            value.get(scope, key.into())
        })
        .and_then(|value| v8::Local::<v8::Object>::try_from(value).ok())
        .map(|value| v8::Global::new(scope, value));
    super::ui_event::attach(
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
    scope
        .get_slot_mut::<FocusEventStore>()
        .expect("FocusEvent state")
        .related_targets
        .insert(arguments.this().get_identity_hash().get(), related_target);
    result.set(arguments.this().into());
}

pub(crate) fn optional_value(
    scope: &mut v8::PinScope<'_, '_>,
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

pub(crate) fn get_related_target(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let related = scope.get_slot::<FocusEventStore>().and_then(|store| {
        store
            .related_targets
            .get(&arguments.this().get_identity_hash().get())
            .cloned()
    });
    let Some(related) = related else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let Some(related) = related else {
        result.set(v8::null(scope).into());
        return;
    };
    let mut related = v8::Local::new(scope, &related);
    let current_target = super::event::record(scope, arguments.this())
        .and_then(|record| record.current_target)
        .map(|target| v8::Local::new(scope, &target));
    if let Some(current_target) = current_target {
        loop {
            if super::node::record(scope, related).is_none() {
                break;
            }
            let root = super::node::root_node(scope, related);
            let Some(host) = super::shadow_root::host(scope, root) else {
                break;
            };
            if shadow_including_contains(scope, root, current_target) {
                break;
            }
            related = host;
        }
    }
    result.set(related.into());
}

fn shadow_including_contains(
    scope: &v8::PinScope<'_, '_>,
    ancestor: v8::Local<'_, v8::Object>,
    descendant: v8::Local<'_, v8::Object>,
) -> bool {
    let mut current = Some(descendant);
    while let Some(node) = current {
        if node.get_identity_hash() == ancestor.get_identity_hash() {
            return true;
        }
        current =
            super::node::parent(scope, node).or_else(|| super::shadow_root::host(scope, node));
    }
    false
}
