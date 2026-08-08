use std::collections::HashMap;

#[derive(Clone, Default)]
struct SpeechGrammarListRecord {
    grammars: Vec<v8::Global<v8::Object>>,
}

#[derive(Default)]
pub(crate) struct SpeechGrammarListStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, SpeechGrammarListRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(SpeechGrammarListStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "SpeechGrammarList", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<SpeechGrammarListStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "SpeechGrammarList",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "length", get_length)?;
    crate::webidl::define_method(scope, prototype, "addFromString", 1, add_from_string)?;
    crate::webidl::define_method(scope, prototype, "addFromUri", 1, add_from_uri)?;
    crate::webidl::define_method(scope, prototype, "item", 1, item)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::define_indexed_iterator(scope, prototype)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<SpeechGrammarListStore>()
        .ok_or_else(|| "SpeechGrammarList state was not prepared".to_owned())?
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
            "Failed to construct 'SpeechGrammarList': Please use the 'new' operator.",
        );
        return;
    }
    attach(scope, arguments.this());
    result.set(arguments.this().into());
}

fn attach(scope: &mut v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) {
    scope
        .get_slot_mut::<SpeechGrammarListStore>()
        .expect("SpeechGrammarList state")
        .records
        .insert(
            object.get_identity_hash().get(),
            SpeechGrammarListRecord::default(),
        );
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create SpeechGrammarList".to_owned());
    }
    attach(scope, object);
    Ok(object)
}

pub(crate) fn is_instance(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> bool {
    scope
        .get_slot::<SpeechGrammarListStore>()
        .is_some_and(|store| {
            store
                .records
                .contains_key(&object.get_identity_hash().get())
        })
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<SpeechGrammarListRecord> {
    scope
        .get_slot::<SpeechGrammarListStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_length(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new_from_unsigned(scope, record.grammars.len() as u32).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn item(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let index = arguments.get(0).integer_value(scope).unwrap_or(-1);
    if index < 0 {
        result.set(v8::null(scope).into());
        return;
    }
    match record.grammars.get(index as usize) {
        Some(grammar) => result.set(v8::Local::new(scope, grammar).into()),
        None => result.set(v8::null(scope).into()),
    }
}

fn add_from_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'addFromString' on 'SpeechGrammarList': 1 argument required, but only 0 present.",
        );
        return;
    }
    let source = crate::webidl::value_to_string(scope, arguments.get(0));
    let weight = optional_weight(scope, arguments.get(1));
    let src = format!("data:application/xml,{source}");
    add(scope, arguments.this(), src, weight);
}

fn add_from_uri(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'addFromUri' on 'SpeechGrammarList': 1 argument required, but only 0 present.",
        );
        return;
    }
    let input = crate::webidl::value_to_string(scope, arguments.get(0));
    let src = super::speech_grammar::resolve_url(scope, &input);
    let weight = optional_weight(scope, arguments.get(1));
    add(scope, arguments.this(), src, weight);
}

fn optional_weight(scope: &mut v8::PinScope<'_, '_>, value: v8::Local<'_, v8::Value>) -> f32 {
    if value.is_undefined() {
        1.0
    } else {
        value.number_value(scope).unwrap_or(1.0) as f32
    }
}

fn add(
    scope: &mut v8::PinScope<'_, '_>,
    list: v8::Local<'_, v8::Object>,
    src: String,
    weight: f32,
) {
    let Ok(grammar) = super::speech_grammar::create(scope, src, weight) else {
        return;
    };
    let grammar_global = v8::Global::new(scope, grammar);
    let index = {
        let Some(record) = scope
            .get_slot_mut::<SpeechGrammarListStore>()
            .and_then(|store| store.records.get_mut(&list.get_identity_hash().get()))
        else {
            crate::webidl::throw_type_error(scope, "Illegal invocation");
            return;
        };
        let index = record.grammars.len() as u32;
        record.grammars.push(grammar_global);
        index
    };
    if let Some(key) = v8::String::new(scope, &index.to_string()) {
        let _ = list.define_own_property(
            scope,
            key.into(),
            grammar.into(),
            v8::PropertyAttribute::READ_ONLY | v8::PropertyAttribute::DONT_DELETE,
        );
    }
}
