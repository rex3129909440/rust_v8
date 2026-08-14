use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct HtmlOptionsCollectionStore {
    constructor: crate::webidl::RealmConstructor,
    owners: HashMap<i32, v8::Global<v8::Object>>,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(HtmlOptionsCollectionStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "HTMLOptionsCollection", c.into())
}
fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(c) = scope
        .get_slot::<HtmlOptionsCollectionStore>()
        .and_then(|s| s.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &c));
    }
    let parent = super::html_collection::ensure_constructor(scope)?;
    let c = crate::webidl::create_function(
        scope,
        "HTMLOptionsCollection",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    crate::webidl::inherit(scope, c, parent)?;
    let p = crate::webidl::prototype(scope, c)?;
    crate::webidl::reset_constructor_order(scope, p)?;
    crate::webidl::define_accessor(scope, p, "length", get_length, set_length)?;
    crate::webidl::define_accessor(
        scope,
        p,
        "selectedIndex",
        get_selected_index,
        set_selected_index,
    )?;
    crate::webidl::define_method(scope, p, "add", 1, add)?;
    crate::webidl::define_method(scope, p, "remove", 1, remove)?;
    crate::webidl::finish_constructor(scope, p, c)?;
    crate::webidl::define_indexed_iterator(scope, p)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, c);
    scope
        .get_slot_mut::<HtmlOptionsCollectionStore>()
        .ok_or_else(|| "HTMLOptionsCollection state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    owner: v8::Local<'s, v8::Object>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure_constructor(scope)?;
    let p = crate::webidl::prototype(scope, c)?;
    let collection = super::html_collection::create(scope, Vec::new())?;
    if crate::webidl::set_platform_prototype(scope, collection, p.into()) != Some(true) {
        return Err("cannot create HTMLOptionsCollection".to_owned());
    }
    let owner = v8::Global::new(scope, owner);
    scope
        .get_slot_mut::<HtmlOptionsCollectionStore>()
        .ok_or_else(|| "HTMLOptionsCollection state was not prepared".to_owned())?
        .owners
        .insert(collection.get_identity_hash().get(), owner);
    Ok(collection)
}
pub(crate) fn refresh(
    scope: &mut v8::PinScope<'_, '_>,
    collection: v8::Local<'_, v8::Object>,
    items: Vec<v8::Local<'_, v8::Object>>,
) -> bool {
    super::html_collection::replace(scope, collection, items)
}
fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor")
}
fn owner<'s>(
    scope: &v8::PinScope<'s, '_>,
    collection: v8::Local<'s, v8::Object>,
) -> Option<v8::Local<'s, v8::Object>> {
    scope
        .get_slot::<HtmlOptionsCollectionStore>()?
        .owners
        .get(&collection.get_identity_hash().get())
        .map(|v| v8::Local::new(scope, v))
}

pub(crate) fn is_options_collection(
    scope: &v8::PinScope<'_, '_>,
    collection: v8::Local<'_, v8::Object>,
) -> bool {
    scope
        .get_slot::<HtmlOptionsCollectionStore>()
        .is_some_and(|store| {
            store
                .owners
                .contains_key(&collection.get_identity_hash().get())
        })
}

pub(crate) fn set_indexed_value(
    scope: &mut v8::PinScope<'_, '_>,
    collection: v8::Local<'_, v8::Object>,
    index: usize,
    value: v8::Local<'_, v8::Value>,
) -> bool {
    let Some(owner) = owner(scope, collection) else {
        return false;
    };
    let current = super::html_select_element::options_snapshot(scope, owner);
    if value.is_null_or_undefined() {
        if let Some(option) = current.get(index) {
            let _ = super::node::detach(scope, *option);
            super::html_select_element::refresh(scope, owner);
        }
        return true;
    }
    let Ok(option) = v8::Local::<v8::Object>::try_from(value) else {
        crate::webidl::throw_type_error(
            scope,
            &format!(
                "Failed to set an indexed property [{index}] on 'HTMLOptionsCollection': parameter 2 is not of type 'HTMLOptionElement'."
            ),
        );
        return true;
    };
    if !super::html_option_element::is_option(scope, option) {
        crate::webidl::throw_type_error(
            scope,
            &format!(
                "Failed to set an indexed property [{index}] on 'HTMLOptionsCollection': parameter 2 is not of type 'HTMLOptionElement'."
            ),
        );
        return true;
    }
    if index < current.len() {
        let _ = super::node::detach(scope, current[index]);
    } else {
        for _ in current.len()..index {
            let Ok(blank) = super::html_option_element::create(
                scope,
                String::new(),
                String::new(),
                false,
                false,
            ) else {
                return true;
            };
            let insertion = super::node::children(scope, owner).len();
            let _ = super::node::insert_child(scope, owner, blank, insertion);
        }
    }
    let insertion = if index >= current.len() {
        super::node::children(scope, owner).len()
    } else {
        index.min(super::node::children(scope, owner).len())
    };
    let _ = super::node::insert_child(scope, owner, option, insertion);
    super::html_select_element::refresh(scope, owner);
    true
}
fn get_length(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(owner) = owner(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let length = super::html_select_element::options_snapshot(scope, owner).len() as u32;
    r.set(v8::Integer::new_from_unsigned(scope, length).into())
}
fn set_length(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let requested = a.get(0).uint32_value(scope).unwrap_or(0) as usize;
    let Some(owner) = owner(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let current = super::html_select_element::options_snapshot(scope, owner);
    if requested < current.len() {
        for option in current[requested..].iter().rev() {
            let _ = super::node::detach(scope, *option);
        }
    } else {
        for _ in current.len()..requested {
            match super::html_option_element::create(
                scope,
                String::new(),
                String::new(),
                false,
                false,
            ) {
                Ok(option) => {
                    let index = super::node::children(scope, owner).len();
                    let _ = super::node::insert_child(scope, owner, option, index);
                }
                Err(message) => {
                    crate::webidl::throw_type_error(scope, &message);
                    return;
                }
            }
        }
    }
    super::html_select_element::refresh(scope, owner)
}
fn get_selected_index(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(owner) = owner(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    r.set(
        v8::Integer::new(
            scope,
            super::html_select_element::selected_index(scope, owner),
        )
        .into(),
    )
}
fn set_selected_index(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let index = a.get(0).int32_value(scope).unwrap_or(-1);
    let Some(owner) = owner(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    super::html_select_element::set_selected_index_value(scope, owner, index)
}
fn add(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(owner) = owner(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    super::html_select_element::add_option_value(scope, owner, a.get(0), a.get(1))
}
fn remove(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(owner) = owner(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let index = a.get(0).int32_value(scope).unwrap_or(-1);
    super::html_select_element::remove_option_index(scope, owner, index)
}
