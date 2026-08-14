use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct MediaElementAudioSourceNodeStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, v8::Global<v8::Object>>,
    connected_elements: HashMap<i32, i32>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(MediaElementAudioSourceNodeStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "MediaElementAudioSourceNode", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<MediaElementAudioSourceNodeStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "MediaElementAudioSourceNode",
        2,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "mediaElement", get_media_element)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::audio_node::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<MediaElementAudioSourceNodeStore>()
        .ok_or_else(|| "MediaElementAudioSourceNode state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 2 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'MediaElementAudioSourceNode': 2 arguments required",
        );
        return;
    }
    let Ok(context) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'MediaElementAudioSourceNode': parameter 1 is not of type 'AudioContext'.",
        );
        return;
    };
    if !super::base_audio_context::is_context(scope, context)
        || super::offline_audio_context::is_offline_context(scope, context)
    {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'MediaElementAudioSourceNode': parameter 1 is not of type 'AudioContext'.",
        );
        return;
    }
    let Ok(options) = v8::Local::<v8::Object>::try_from(arguments.get(1)) else {
        crate::webidl::throw_type_error(scope, "The options argument must be an object");
        return;
    };
    let Some(key) = v8::String::new(scope, "mediaElement") else {
        return;
    };
    let Some(media_value) = options.get(scope, key.into()) else {
        return;
    };
    let Ok(media_element) = v8::Local::<v8::Object>::try_from(media_value) else {
        crate::webidl::throw_type_error(scope, "mediaElement is required");
        return;
    };
    if !super::html_media_element::is_media_element(scope, media_element) {
        crate::webidl::throw_type_error(scope, "mediaElement is not an HTMLMediaElement");
        return;
    }
    let media_id = media_element.get_identity_hash().get();
    if scope
        .get_slot::<MediaElementAudioSourceNodeStore>()
        .is_some_and(|store| store.connected_elements.contains_key(&media_id))
    {
        throw_invalid_state(scope);
        return;
    }
    super::audio_node::attach(scope, arguments.this(), Some(context), 0, 1);
    let media_element_global = v8::Global::new(scope, media_element);
    let node_id = arguments.this().get_identity_hash().get();
    let store = scope
        .get_slot_mut::<MediaElementAudioSourceNodeStore>()
        .expect("MediaElementAudioSourceNode state");
    store.records.insert(node_id, media_element_global);
    store.connected_elements.insert(media_id, node_id);
    result.set(arguments.this().into());
}

fn get_media_element(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(media_element) = scope
        .get_slot::<MediaElementAudioSourceNodeStore>()
        .and_then(|store| {
            store
                .records
                .get(&arguments.this().get_identity_hash().get())
        })
    {
        result.set(v8::Local::new(scope, media_element).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn throw_invalid_state(scope: &mut v8::PinScope<'_, '_>) {
    match super::dom_exception::create(
        scope,
        "Failed to construct 'MediaElementAudioSourceNode': HTMLMediaElement already connected previously to a different MediaElementSourceNode.".to_owned(),
        "InvalidStateError".to_owned(),
    ) {
        Ok(exception) => {
            scope.throw_exception(exception.into());
        }
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
