use std::collections::HashMap;
#[derive(Clone)]
struct RequestRecord {
    id: String,
    shipping: v8::Global<v8::Object>,
    shipping_option: Option<String>,
    shipping_type: Option<String>,
    address_handler: Option<v8::Global<v8::Value>>,
    option_handler: Option<v8::Global<v8::Value>>,
    method_handler: Option<v8::Global<v8::Value>>,
    aborted: bool,
}
#[derive(Default)]
pub(crate) struct PaymentRequestStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, RequestRecord>,
    next: u64,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(PaymentRequestStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "PaymentRequest", c.into())
}
fn ensure<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(v) = s
        .get_slot::<PaymentRequestStore>()
        .and_then(|x| x.constructor.get(crate::webidl::realm_id(s)))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c = crate::webidl::create_function(
        s,
        "PaymentRequest",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_readonly_accessor(s, p, "id", id)?;
    crate::webidl::define_readonly_accessor(s, p, "shippingAddress", shipping_address)?;
    crate::webidl::define_readonly_accessor(s, p, "shippingOption", shipping_option)?;
    crate::webidl::define_readonly_accessor(s, p, "shippingType", shipping_type)?;
    crate::webidl::define_accessor(
        s,
        p,
        "onshippingaddresschange",
        get_address_handler,
        set_address_handler,
    )?;
    crate::webidl::define_accessor(
        s,
        p,
        "onshippingoptionchange",
        get_option_handler,
        set_option_handler,
    )?;
    crate::webidl::define_method(s, p, "abort", 0, abort)?;
    crate::webidl::define_method(s, p, "canMakePayment", 0, can_make_payment)?;
    crate::webidl::define_method(s, p, "hasEnrolledInstrument", 0, has_enrolled_instrument)?;
    crate::webidl::define_method(s, p, "show", 0, show)?;
    crate::webidl::define_accessor(
        s,
        p,
        "onpaymentmethodchange",
        get_method_handler,
        set_method_handler,
    )?;
    crate::webidl::finish_constructor(s, p, c)?;
    let parent = super::event_target::ensure_constructor(s)?;
    crate::webidl::inherit(s, c, parent)?;
    crate::webidl::define_method(
        s,
        c.into(),
        "getSecurePaymentConfirmationCapabilities",
        0,
        capabilities,
    )?;
    crate::webidl::define_method(
        s,
        c.into(),
        "securePaymentConfirmationAvailability",
        0,
        availability,
    )?;
    let realm_id = crate::webidl::realm_id(s);
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<PaymentRequestStore>()
        .unwrap()
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
fn construct(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !a.is_construct_call() || a.length() < 1 {
        crate::webidl::throw_type_error(s, "method data required");
        return;
    }
    if !a.get(0).is_object() {
        crate::webidl::throw_type_error(
            s,
            "Failed to construct 'PaymentRequest': The provided value cannot be converted to a sequence.",
        );
        return;
    }
    let method_data = match crate::webidl::sequence_values(s, a.get(0)) {
        Ok(method_data) => method_data,
        Err(_) => {
            crate::webidl::throw_type_error(
                s,
                "Failed to construct 'PaymentRequest': The object must have a callable @@iterator property.",
            );
            return;
        }
    };
    if method_data.is_empty() {
        crate::webidl::throw_type_error(
            s,
            "Failed to construct 'PaymentRequest': At least one payment method is required",
        );
        return;
    }
    for method in &method_data {
        let method = v8::Local::new(s, method);
        if !method.is_object() {
            crate::webidl::throw_type_error(
                s,
                "Failed to construct 'PaymentRequest': The provided value is not of type 'PaymentMethodData'.",
            );
            return;
        }
    }
    super::event_target::attach(s, a.this());
    let address = match super::payment_address::create(s, Default::default()) {
        Ok(v) => v,
        Err(e) => {
            crate::webidl::throw_type_error(s, &e);
            return;
        }
    };
    let shipping = v8::Global::new(s, address);
    let next = {
        let store = s.get_slot_mut::<PaymentRequestStore>().unwrap();
        store.next += 1;
        store.next
    };
    let id = format!("payment-request-{next}");
    s.get_slot_mut::<PaymentRequestStore>()
        .unwrap()
        .records
        .insert(
            a.this().get_identity_hash().get(),
            RequestRecord {
                id,
                shipping,
                shipping_option: None,
                shipping_type: None,
                address_handler: None,
                option_handler: None,
                method_handler: None,
                aborted: false,
            },
        );
    r.set(a.this().into())
}
fn record(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<RequestRecord> {
    s.get_slot::<PaymentRequestStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
fn promise(s: &mut v8::PinScope<'_, '_>, v: v8::Local<'_, v8::Value>, mut r: v8::ReturnValue<'_>) {
    if let Ok(p) = super::writable_stream::resolved_promise(s, v) {
        r.set(p.into())
    }
}
fn id(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this())
        && let Some(x) = v8::String::new(s, &v.id)
    {
        r.set(x.into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn shipping_address(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Local::new(s, &v.shipping).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn optional(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
    f: impl FnOnce(RequestRecord) -> Option<String>,
) {
    if let Some(v) = record(s, a.this()) {
        if let Some(x) = f(v).and_then(|x| v8::String::new(s, &x)) {
            r.set(x.into())
        } else {
            r.set(v8::null(s).into())
        }
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn shipping_option(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    optional(s, a, r, |v| v.shipping_option)
}
fn shipping_type(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    optional(s, a, r, |v| v.shipping_type)
}
fn handler(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
    which: u8,
) {
    let Some(record) = record(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    let h = match which {
        0 => record.address_handler,
        1 => record.option_handler,
        _ => record.method_handler,
    };
    super::window_event_handler_support::return_handler(s, h, r)
}
fn get_address_handler(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    handler(s, a, r, 0)
}
fn get_option_handler(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    handler(s, a, r, 1)
}
fn get_method_handler(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    handler(s, a, r, 2)
}
fn set_handler(s: &mut v8::PinScope<'_, '_>, a: v8::FunctionCallbackArguments<'_>, which: u8) {
    let h = super::window_event_handler_support::handler_value(s, a.get(0));
    if let Some(v) = s
        .get_slot_mut::<PaymentRequestStore>()
        .and_then(|x| x.records.get_mut(&a.this().get_identity_hash().get()))
    {
        match which {
            0 => v.address_handler = h,
            1 => v.option_handler = h,
            _ => v.method_handler = h,
        }
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn set_address_handler(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, 0)
}
fn set_option_handler(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, 1)
}
fn set_method_handler(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, 2)
}
fn abort(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = s
        .get_slot_mut::<PaymentRequestStore>()
        .and_then(|x| x.records.get_mut(&a.this().get_identity_hash().get()))
    {
        v.aborted = true;
        let x = v8::undefined(s);
        promise(s, x.into(), r)
    } else {
        crate::webidl::reject_illegal_invocation_promise(s, "PaymentRequest", "abort", r)
    }
}
fn true_promise(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
    method_name: &str,
) {
    if record(s, a.this()).is_none() {
        crate::webidl::reject_illegal_invocation_promise(s, "PaymentRequest", method_name, r);
        return;
    }
    let x = v8::Boolean::new(s, true);
    promise(s, x.into(), r)
}
fn can_make_payment(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    true_promise(s, a, r, "canMakePayment")
}
fn has_enrolled_instrument(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    true_promise(s, a, r, "hasEnrolledInstrument")
}
fn show(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let Some(v) = record(s, a.this()) else {
        crate::webidl::reject_illegal_invocation_promise(s, "PaymentRequest", "show", r);
        return;
    };
    match super::payment_response::create(s, v.id) {
        Ok(x) => promise(s, x.into(), r),
        Err(e) => crate::webidl::throw_type_error(s, &e),
    }
}
fn capabilities(
    s: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let o = v8::Object::new(s);
    promise(s, o.into(), r)
}
fn availability(
    s: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let x = v8::String::new(s, "available").unwrap();
    promise(s, x.into(), r)
}
