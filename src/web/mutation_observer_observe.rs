use super::mutation_observer::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "observe", 1, observe)
}

fn observe(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if arguments.length() < 2 {
        crate::webidl::throw_type_error(scope, "Failed to execute 'observe': 2 arguments required");
        return;
    }
    let Ok(target) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "MutationObserver target is not a Node");
        return;
    };
    if super::node::record(scope, target).is_none() {
        crate::webidl::throw_type_error(scope, "MutationObserver target is not a Node");
        return;
    }
    let Ok(options) = v8::Local::<v8::Object>::try_from(arguments.get(1)) else {
        crate::webidl::throw_type_error(scope, "MutationObserver options must be an object");
        return;
    };
    let attribute_old_value = boolean_property(scope, options, "attributeOldValue");
    let character_data_old_value = boolean_property(scope, options, "characterDataOldValue");
    let attribute_filter = string_sequence_property(scope, options, "attributeFilter");
    let attributes = optional_boolean_property(scope, options, "attributes")
        .unwrap_or(attribute_old_value || attribute_filter.is_some());
    let character_data = optional_boolean_property(scope, options, "characterData")
        .unwrap_or(character_data_old_value);
    let child_list = boolean_property(scope, options, "childList");
    let subtree = boolean_property(scope, options, "subtree");
    if !attributes && !character_data && !child_list {
        crate::webidl::throw_type_error(
            scope,
            "The options must enable attributes, characterData, or childList",
        );
        return;
    }
    if !attributes && (attribute_old_value || attribute_filter.is_some()) {
        crate::webidl::throw_type_error(scope, "attributes is false");
        return;
    }
    if !character_data && character_data_old_value {
        crate::webidl::throw_type_error(scope, "characterData is false");
        return;
    }
    let observed = ObservedTarget {
        target: v8::Global::new(scope, target),
        child_list,
        attributes,
        character_data,
        subtree,
        attribute_old_value,
        character_data_old_value,
        attribute_filter,
    };
    let id = arguments.this().get_identity_hash().get();
    let target_id = target.get_identity_hash().get();
    let existing_index = scope
        .get_slot::<MutationObserverStore>()
        .and_then(|store| store.observers.get(&id))
        .and_then(|observer| {
            observer.observed_targets.iter().position(|current| {
                v8::Local::new(scope, &current.target)
                    .get_identity_hash()
                    .get()
                    == target_id
            })
        });
    let Some(observer) = scope
        .get_slot_mut::<MutationObserverStore>()
        .and_then(|store| store.observers.get_mut(&id))
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(index) = existing_index {
        observer.observed_targets[index] = observed;
    } else {
        observer.observed_targets.push(observed);
    }
}
