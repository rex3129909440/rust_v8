use super::html_details_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "open", get_open, set_open)
}

fn get_open(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, record.open).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_open(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let next = arguments.get(0).boolean_value(scope);
    let identity = arguments.this().get_identity_hash().get();
    let should_queue = scope
        .get_slot_mut::<HtmlDetailsElementStore>()
        .and_then(|store| store.records.get_mut(&identity))
        .map(|record| {
            if record.open == next {
                return false;
            }
            record.open = next;
            if record.toggle_pending {
                false
            } else {
                record.toggle_pending = true;
                true
            }
        });
    let Some(should_queue) = should_queue else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if next {
        super::element::set_attribute_full(
            scope,
            arguments.this(),
            "open".to_owned(),
            String::new(),
            None,
        );
    } else {
        super::element::remove_attribute_full(scope, arguments.this(), None, "open");
    }
    if should_queue {
        let data = v8::Integer::new(scope, identity);
        if let Some(callback) = v8::Function::builder(deliver_toggle)
            .data(data.into())
            .length(0)
            .constructor_behavior(v8::ConstructorBehavior::Throw)
            .build(scope)
        {
            scope.enqueue_microtask(callback);
        }
    }
}
