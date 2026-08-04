use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct TextStore {
    constructors: HashMap<i32, v8::Global<v8::Function>>,
    instances: HashMap<i32, ()>,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(TextStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "Text", c.into())
}
pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<TextStore>()
        .and_then(|s| s.constructors.get(&crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(e) = existing {
        return Ok(v8::Local::new(scope, &e));
    }
    let c = crate::webidl::create_function(
        scope,
        "Text",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let p = crate::webidl::prototype(scope, c)?;
    crate::webidl::reset_constructor_order(scope, p)?;
    super::text_whole_text_property::define(scope, p)?;
    super::text_assigned_slot::define(scope, p)?;
    super::text_split_text::define(scope, p)?;
    crate::webidl::finish_constructor(scope, p, c)?;
    let parent = super::character_data::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, c, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let stored = v8::Global::new(scope, c);
    scope
        .get_slot_mut::<TextStore>()
        .ok_or_else(|| "Text state was not prepared".to_owned())?
        .constructors
        .insert(realm_id, stored);
    Ok(c)
}
fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !a.is_construct_call() {
        crate::webidl::throw_type_error(scope, "Failed to construct 'Text': use new");
        return;
    }
    let data = if a.length() == 0 {
        String::new()
    } else {
        crate::webidl::value_to_string(scope, a.get(0))
    };
    attach(scope, a.this(), data);
    r.set(a.this().into())
}
pub(crate) fn attach(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    data: String,
) {
    super::node::attach(scope, object, 3, "#text".to_owned(), Some(data.clone()));
    super::character_data::attach(scope, object, data);
    scope
        .get_slot_mut::<TextStore>()
        .expect("Text state")
        .instances
        .insert(object.get_identity_hash().get(), ());
}
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    data: String,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure_constructor(scope)?;
    let p = crate::webidl::prototype(scope, c)?;
    let o = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, o, p.into()) != Some(true) {
        return Err("cannot create Text".to_owned());
    }
    attach(scope, o, data);
    Ok(o)
}
pub(crate) fn data_if_text(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<String> {
    if scope
        .get_slot::<TextStore>()
        .is_some_and(|s| s.instances.contains_key(&object.get_identity_hash().get()))
    {
        super::character_data::data_if_character(scope, object)
    } else {
        None
    }
}
