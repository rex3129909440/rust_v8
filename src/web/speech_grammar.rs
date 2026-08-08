use std::collections::HashMap;

#[derive(Clone)]
struct SpeechGrammarRecord {
    src: String,
    weight: f32,
}

#[derive(Default)]
pub(crate) struct SpeechGrammarStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, SpeechGrammarRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(SpeechGrammarStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "SpeechGrammar", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<SpeechGrammarStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "SpeechGrammar",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_accessor(scope, prototype, "src", get_src, set_src)?;
    crate::webidl::define_accessor(scope, prototype, "weight", get_weight, set_weight)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<SpeechGrammarStore>()
        .ok_or_else(|| "SpeechGrammar state was not prepared".to_owned())?
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
            "Failed to construct 'SpeechGrammar': Please use the 'new' operator.",
        );
        return;
    }
    attach(scope, arguments.this(), String::new(), 1.0);
    result.set(arguments.this().into());
}

fn attach(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    src: String,
    weight: f32,
) {
    scope
        .get_slot_mut::<SpeechGrammarStore>()
        .expect("SpeechGrammar state")
        .records
        .insert(
            object.get_identity_hash().get(),
            SpeechGrammarRecord { src, weight },
        );
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    src: String,
    weight: f32,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create SpeechGrammar".to_owned());
    }
    attach(scope, object, src, weight);
    Ok(object)
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<SpeechGrammarRecord> {
    scope
        .get_slot::<SpeechGrammarStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_src(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(value) = v8::String::new(scope, &record.src) {
        result.set(value.into());
    }
}

fn set_src(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let input = crate::webidl::value_to_string(scope, arguments.get(0));
    let src = resolve_url(scope, &input);
    let Some(record) = scope
        .get_slot_mut::<SpeechGrammarStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    record.src = src;
}

fn get_weight(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, record.weight as f64).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_weight(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let weight = arguments.get(0).number_value(scope).unwrap_or(1.0) as f32;
    let Some(record) = scope
        .get_slot_mut::<SpeechGrammarStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    record.weight = weight;
}

pub(crate) fn resolve_url(scope: &mut v8::PinScope<'_, '_>, input: &str) -> String {
    if let Ok(url) = url::Url::parse(input) {
        return url.to_string();
    }
    let global = scope.get_current_context().global(scope);
    let base = v8::String::new(scope, "location")
        .and_then(|key| global.get(scope, key.into()))
        .and_then(|value| v8::Local::<v8::Object>::try_from(value).ok())
        .and_then(|location| {
            v8::String::new(scope, "href")
                .and_then(|key| location.get(scope, key.into()))
                .map(|value| crate::webidl::value_to_string(scope, value))
        })
        .unwrap_or_else(|| "about:blank".to_owned());
    url::Url::parse(&base)
        .and_then(|url| url.join(input))
        .map(|url| url.to_string())
        .unwrap_or_default()
}
