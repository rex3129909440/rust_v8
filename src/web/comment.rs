use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct CommentStore {
    constructors: HashMap<i32, v8::Global<v8::Function>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CommentStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "Comment", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<CommentStore>()
        .and_then(|store| store.constructors.get(&crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "Comment",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::character_data::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<CommentStore>()
        .ok_or_else(|| "Comment state was not prepared".to_owned())?
        .constructors
        .insert(realm_id, stored);
    Ok(constructor)
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(scope, "Comment requires new");
        return;
    }
    let data = if arguments.get(0).is_undefined() {
        String::new()
    } else {
        crate::webidl::value_to_string(scope, arguments.get(0))
    };
    super::node::attach(
        scope,
        arguments.this(),
        8,
        "#comment".to_owned(),
        Some(data.clone()),
    );
    super::character_data::attach(scope, arguments.this(), data);
    result.set(arguments.this().into());
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    data: String,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let comment = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, comment, prototype.into()) != Some(true) {
        return Err("cannot create Comment".to_owned());
    }
    super::node::attach(scope, comment, 8, "#comment".to_owned(), Some(data.clone()));
    super::character_data::attach(scope, comment, data);
    Ok(comment)
}
