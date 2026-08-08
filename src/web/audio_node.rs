use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct AudioNodeStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, AudioNodeRecord>,
}

#[derive(Clone)]
struct AudioNodeRecord {
    object: v8::Global<v8::Object>,
    context: Option<v8::Global<v8::Object>>,
    number_of_inputs: u32,
    number_of_outputs: u32,
    channel_count: u32,
    channel_count_mode: String,
    channel_interpretation: String,
    connections: Vec<AudioConnection>,
}

#[derive(Clone)]
struct AudioConnection {
    destination_identity: i32,
    destination: v8::Global<v8::Object>,
    destination_is_param: bool,
    output: u32,
    input: Option<u32>,
}

#[derive(Clone)]
pub(crate) struct AudioConnectionSnapshot {
    pub(crate) destination: v8::Global<v8::Object>,
    pub(crate) destination_is_param: bool,
    pub(crate) output: u32,
    pub(crate) input: Option<u32>,
}

#[derive(Clone)]
pub(crate) struct IncomingAudioConnection {
    pub(crate) source: v8::Global<v8::Object>,
    pub(crate) output: u32,
    pub(crate) input: u32,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(AudioNodeStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "AudioNode", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<AudioNodeStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "AudioNode",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "context", get_context)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "numberOfInputs",
        get_number_of_inputs,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "numberOfOutputs",
        get_number_of_outputs,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "channelCount",
        get_channel_count,
        set_channel_count,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "channelCountMode",
        get_channel_count_mode,
        set_channel_count_mode,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "channelInterpretation",
        get_channel_interpretation,
        set_channel_interpretation,
    )?;
    crate::webidl::define_method(scope, prototype, "connect", 1, connect)?;
    crate::webidl::define_method(scope, prototype, "disconnect", 0, disconnect)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let event_target = super::event_target::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, event_target)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<AudioNodeStore>()
        .ok_or_else(|| "AudioNode state was not prepared".to_owned())?
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
        "Failed to construct 'AudioNode': Illegal constructor",
    );
}

pub(crate) fn attach(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    context: Option<v8::Local<'_, v8::Object>>,
    number_of_inputs: u32,
    number_of_outputs: u32,
) {
    super::event_target::attach(scope, object);
    let stored_object = v8::Global::new(scope, object);
    let context = context.map(|context| v8::Global::new(scope, context));
    if let Some(store) = scope.get_slot_mut::<AudioNodeStore>() {
        store.records.insert(
            object.get_identity_hash().get(),
            AudioNodeRecord {
                object: stored_object,
                context,
                number_of_inputs,
                number_of_outputs,
                channel_count: 2,
                channel_count_mode: "max".to_owned(),
                channel_interpretation: "speakers".to_owned(),
                connections: Vec::new(),
            },
        );
    }
}

pub(crate) fn set_channel_configuration(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    channel_count: u32,
    channel_count_mode: String,
    channel_interpretation: String,
) -> bool {
    if let Some(record) = scope
        .get_slot_mut::<AudioNodeStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        record.channel_count = channel_count;
        record.channel_count_mode = channel_count_mode;
        record.channel_interpretation = channel_interpretation;
        true
    } else {
        false
    }
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<AudioNodeRecord> {
    scope
        .get_slot::<AudioNodeStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn is_node(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> bool {
    scope.get_slot::<AudioNodeStore>().is_some_and(|store| {
        store
            .records
            .contains_key(&object.get_identity_hash().get())
    })
}

pub(crate) fn context_identity(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<i32> {
    let record = record(scope, object)?;
    record
        .context
        .map(|context| v8::Local::new(scope, &context).get_identity_hash().get())
}

pub(crate) fn context<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<v8::Local<'s, v8::Object>> {
    let context = record(scope, object)?.context?;
    Some(v8::Local::new(scope, &context))
}

pub(crate) fn connections(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Vec<AudioConnectionSnapshot> {
    record(scope, object)
        .map(|record| {
            record
                .connections
                .into_iter()
                .map(|connection| AudioConnectionSnapshot {
                    destination: connection.destination,
                    destination_is_param: connection.destination_is_param,
                    output: connection.output,
                    input: connection.input,
                })
                .collect()
        })
        .unwrap_or_default()
}

pub(crate) fn incoming_connections(
    scope: &v8::PinScope<'_, '_>,
    destination: v8::Local<'_, v8::Object>,
) -> Vec<IncomingAudioConnection> {
    let destination_identity = destination.get_identity_hash().get();
    scope
        .get_slot::<AudioNodeStore>()
        .map(|store| {
            store
                .records
                .values()
                .flat_map(|record| {
                    record.connections.iter().filter_map(|connection| {
                        (connection.destination_identity == destination_identity
                            && !connection.destination_is_param)
                            .then(|| IncomingAudioConnection {
                                source: record.object.clone(),
                                output: connection.output,
                                input: connection.input.unwrap_or(0),
                            })
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn update(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut AudioNodeRecord),
) {
    if let Some(record) = scope
        .get_slot_mut::<AudioNodeStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        change(record);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_context(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(context) = record.context {
        result.set(v8::Local::new(scope, &context).into());
    } else {
        result.set(v8::null(scope).into());
    }
}

fn return_u32(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&AudioNodeRecord) -> u32,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new_from_unsigned(scope, select(&record)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_number_of_inputs(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_u32(s, a, r, |record| record.number_of_inputs)
}
fn get_number_of_outputs(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_u32(s, a, r, |record| record.number_of_outputs)
}
fn get_channel_count(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_u32(s, a, r, |record| record.channel_count)
}

fn set_channel_count(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0).uint32_value(scope).unwrap_or(0);
    if value == 0 {
        crate::webidl::throw_type_error(scope, "channelCount must be greater than zero");
        return;
    }
    update(scope, arguments.this(), |record| {
        record.channel_count = value
    });
}

fn return_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&AudioNodeRecord) -> &str,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, select(&record)) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_channel_count_mode(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |record| &record.channel_count_mode)
}
fn get_channel_interpretation(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |record| &record.channel_interpretation)
}

fn set_channel_count_mode(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    if value != "max" && value != "clamped-max" && value != "explicit" {
        crate::webidl::throw_type_error(scope, "Invalid channelCountMode");
        return;
    }
    update(scope, arguments.this(), |record| {
        record.channel_count_mode = value
    });
}

fn set_channel_interpretation(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    if value != "speakers" && value != "discrete" {
        crate::webidl::throw_type_error(scope, "Invalid channelInterpretation");
        return;
    }
    update(scope, arguments.this(), |record| {
        record.channel_interpretation = value
    });
}

fn connect(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(source) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let Ok(destination) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'connect' on 'AudioNode': parameter 1 is not of type 'AudioNode' or 'AudioParam'.",
        );
        return;
    };
    let destination_node = record(scope, destination);
    let destination_is_param = super::audio_param::is_param(scope, destination);
    if destination_node.is_none() && !destination_is_param {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'connect' on 'AudioNode': parameter 1 is not of type 'AudioNode' or 'AudioParam'.",
        );
        return;
    }
    let output = if arguments.get(1).is_undefined() {
        0
    } else {
        arguments.get(1).uint32_value(scope).unwrap_or(u32::MAX)
    };
    if output >= source.number_of_outputs {
        throw_dom_exception(
            scope,
            "IndexSizeError",
            &format!(
                "Failed to execute 'connect' on 'AudioNode': output index ({output}) exceeds number of outputs ({}).",
                source.number_of_outputs
            ),
        );
        return;
    }
    let input = if destination_is_param {
        None
    } else {
        let value = if arguments.get(2).is_undefined() {
            0
        } else {
            arguments.get(2).uint32_value(scope).unwrap_or(u32::MAX)
        };
        let number_of_inputs = destination_node
            .as_ref()
            .map_or(0, |record| record.number_of_inputs);
        if value >= number_of_inputs {
            throw_dom_exception(
                scope,
                "IndexSizeError",
                &format!(
                    "Failed to execute 'connect' on 'AudioNode': input index ({value}) exceeds number of inputs ({number_of_inputs})."
                ),
            );
            return;
        }
        Some(value)
    };
    let source_context = source
        .context
        .as_ref()
        .map(|context| v8::Local::new(scope, context).get_identity_hash().get());
    let destination_context = if destination_is_param {
        super::audio_param::context_identity(scope, destination)
    } else {
        destination_node
            .as_ref()
            .and_then(|record| record.context.as_ref())
            .map(|context| v8::Local::new(scope, context).get_identity_hash().get())
    };
    if source_context != destination_context {
        throw_dom_exception(
            scope,
            "InvalidAccessError",
            "Failed to execute 'connect' on 'AudioNode': cannot connect to an AudioNode belonging to a different audio context.",
        );
        return;
    }
    let destination_identity = destination.get_identity_hash().get();
    let stored = v8::Global::new(scope, destination);
    update(scope, arguments.this(), |record| {
        let duplicate = record.connections.iter().any(|connection| {
            connection.destination_identity == destination_identity
                && connection.destination_is_param == destination_is_param
                && connection.output == output
                && connection.input == input
        });
        if !duplicate {
            record.connections.push(AudioConnection {
                destination_identity,
                destination: stored,
                destination_is_param,
                output,
                input,
            });
        }
    });
    if !destination_is_param {
        result.set(destination.into());
    }
}

fn disconnect(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(source) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if arguments.get(0).is_undefined() {
        update(scope, arguments.this(), |record| record.connections.clear());
        return;
    }

    let mut destination_identity = None;
    let mut destination_is_param = None;
    let mut output = None;
    let mut input = None;
    if arguments.get(0).is_number() {
        let value = arguments.get(0).uint32_value(scope).unwrap_or(u32::MAX);
        if value >= source.number_of_outputs {
            throw_dom_exception(
                scope,
                "IndexSizeError",
                &format!(
                    "Failed to execute 'disconnect' on 'AudioNode': output index ({value}) exceeds number of outputs ({}).",
                    source.number_of_outputs
                ),
            );
            return;
        }
        output = Some(value);
    } else {
        let Ok(destination) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
            crate::webidl::throw_type_error(
                scope,
                "Failed to execute 'disconnect' on 'AudioNode': invalid destination.",
            );
            return;
        };
        let destination_node = record(scope, destination);
        let is_param = super::audio_param::is_param(scope, destination);
        if destination_node.is_none() && !is_param {
            crate::webidl::throw_type_error(
                scope,
                "Failed to execute 'disconnect' on 'AudioNode': invalid destination.",
            );
            return;
        }
        let source_context = source
            .context
            .as_ref()
            .map(|context| v8::Local::new(scope, context).get_identity_hash().get());
        let destination_context = if is_param {
            super::audio_param::context_identity(scope, destination)
        } else {
            destination_node
                .as_ref()
                .and_then(|record| record.context.as_ref())
                .map(|context| v8::Local::new(scope, context).get_identity_hash().get())
        };
        if source_context != destination_context {
            throw_dom_exception(
                scope,
                "InvalidAccessError",
                "Failed to execute 'disconnect' on 'AudioNode': cannot disconnect from an AudioNode belonging to a different audio context.",
            );
            return;
        }
        destination_identity = Some(destination.get_identity_hash().get());
        destination_is_param = Some(is_param);
        if !arguments.get(1).is_undefined() {
            let value = arguments.get(1).uint32_value(scope).unwrap_or(u32::MAX);
            if value >= source.number_of_outputs {
                throw_dom_exception(
                    scope,
                    "IndexSizeError",
                    &format!(
                        "Failed to execute 'disconnect' on 'AudioNode': output index ({value}) exceeds number of outputs ({}).",
                        source.number_of_outputs
                    ),
                );
                return;
            }
            output = Some(value);
        }
        if !is_param && !arguments.get(2).is_undefined() {
            let value = arguments.get(2).uint32_value(scope).unwrap_or(u32::MAX);
            let number_of_inputs = destination_node
                .as_ref()
                .map_or(0, |record| record.number_of_inputs);
            if value >= number_of_inputs {
                throw_dom_exception(
                    scope,
                    "IndexSizeError",
                    &format!(
                        "Failed to execute 'disconnect' on 'AudioNode': input index ({value}) exceeds number of inputs ({number_of_inputs})."
                    ),
                );
                return;
            }
            input = Some(value);
        }
    }

    let mut removed = false;
    update(scope, arguments.this(), |record| {
        record.connections.retain(|connection| {
            let matches = destination_identity
                .is_none_or(|identity| connection.destination_identity == identity)
                && destination_is_param
                    .is_none_or(|is_param| connection.destination_is_param == is_param)
                && output.is_none_or(|value| connection.output == value)
                && input.is_none_or(|value| connection.input == Some(value));
            removed |= matches;
            !matches
        });
    });
    if !removed {
        throw_dom_exception(
            scope,
            "InvalidAccessError",
            "Failed to execute 'disconnect' on 'AudioNode': the given connection does not exist.",
        );
    }
}

fn throw_dom_exception(scope: &mut v8::PinScope<'_, '_>, name: &str, message: &str) {
    if let Ok(exception) = super::dom_exception::create(scope, message.to_owned(), name.to_owned())
    {
        scope.throw_exception(exception.into());
    }
}
