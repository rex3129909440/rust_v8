use std::collections::HashMap;
#[derive(Clone)]
struct DeviceData {
    opened: bool,
    configuration: v8::Global<v8::Object>,
    configurations: v8::Global<v8::Array>,
    profile: crate::UsbDeviceFingerprint,
}
#[derive(Default)]
pub(crate) struct UsbDeviceStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, DeviceData>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(UsbDeviceStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "USBDevice", c.into())
}
fn ensure<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(s);
    if let Some(v) = s
        .get_slot::<UsbDeviceStore>()
        .and_then(|x| x.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c =
        crate::webidl::create_function(s, "USBDevice", 0, v8::ConstructorBehavior::Allow, illegal)?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_readonly_accessor(s, p, "usbVersionMajor", usb_version_major)?;
    crate::webidl::define_readonly_accessor(s, p, "usbVersionMinor", usb_version_minor)?;
    crate::webidl::define_readonly_accessor(s, p, "usbVersionSubminor", usb_version_subminor)?;
    crate::webidl::define_readonly_accessor(s, p, "deviceClass", device_class)?;
    crate::webidl::define_readonly_accessor(s, p, "deviceSubclass", device_subclass)?;
    crate::webidl::define_readonly_accessor(s, p, "deviceProtocol", device_protocol)?;
    crate::webidl::define_readonly_accessor(s, p, "vendorId", vendor_id)?;
    crate::webidl::define_readonly_accessor(s, p, "productId", product_id)?;
    crate::webidl::define_readonly_accessor(s, p, "deviceVersionMajor", device_version_major)?;
    crate::webidl::define_readonly_accessor(s, p, "deviceVersionMinor", device_version_minor)?;
    crate::webidl::define_readonly_accessor(
        s,
        p,
        "deviceVersionSubminor",
        device_version_subminor,
    )?;
    crate::webidl::define_readonly_accessor(s, p, "manufacturerName", manufacturer)?;
    crate::webidl::define_readonly_accessor(s, p, "productName", product)?;
    crate::webidl::define_readonly_accessor(s, p, "serialNumber", serial)?;
    crate::webidl::define_readonly_accessor(s, p, "configuration", configuration)?;
    crate::webidl::define_readonly_accessor(s, p, "configurations", configurations)?;
    crate::webidl::define_readonly_accessor(s, p, "opened", opened)?;
    crate::webidl::define_method(s, p, "claimInterface", 1, claim_interface)?;
    crate::webidl::define_method(s, p, "clearHalt", 2, void_method)?;
    crate::webidl::define_method(s, p, "close", 0, close)?;
    crate::webidl::define_method(s, p, "controlTransferIn", 2, control_in)?;
    crate::webidl::define_method(s, p, "controlTransferOut", 1, control_out)?;
    crate::webidl::define_method(s, p, "forget", 0, close)?;
    crate::webidl::define_method(s, p, "isochronousTransferIn", 2, iso_in)?;
    crate::webidl::define_method(s, p, "isochronousTransferOut", 3, iso_out)?;
    crate::webidl::define_method(s, p, "open", 0, open)?;
    crate::webidl::define_method(s, p, "releaseInterface", 1, release_interface)?;
    crate::webidl::define_method(s, p, "reset", 0, void_method)?;
    crate::webidl::define_method(s, p, "selectAlternateInterface", 2, void_method)?;
    crate::webidl::define_method(s, p, "selectConfiguration", 1, void_method)?;
    crate::webidl::define_method(s, p, "transferIn", 2, control_in)?;
    crate::webidl::define_method(s, p, "transferOut", 2, control_out)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<UsbDeviceStore>()
        .unwrap()
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
fn illegal(
    s: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(s, "Illegal constructor")
}
pub(crate) fn create<'s>(
    s: &mut v8::PinScope<'s, '_>,
    profile: crate::UsbDeviceFingerprint,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure(s)?;
    let p = crate::webidl::prototype(s, c)?;
    let o = v8::Object::new(s);
    if crate::webidl::set_platform_prototype(s, o, p.into()) != Some(true) {
        return Err("cannot create USBDevice".to_owned());
    }
    let configuration = super::usb_configuration::create(s, 1)?;
    let configurations = v8::Array::new(s, 1);
    let _ = configurations.set_index(s, 0, configuration.into());
    let configuration_global = v8::Global::new(s, configuration);
    let configurations_global = v8::Global::new(s, configurations);
    s.get_slot_mut::<UsbDeviceStore>().unwrap().records.insert(
        o.get_identity_hash().get(),
        DeviceData {
            opened: false,
            configuration: configuration_global,
            configurations: configurations_global,
            profile,
        },
    );
    Ok(o)
}
fn record(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<DeviceData> {
    s.get_slot::<UsbDeviceStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
fn promise(s: &mut v8::PinScope<'_, '_>, v: v8::Local<'_, v8::Value>, mut r: v8::ReturnValue<'_>) {
    if let Ok(p) = super::writable_stream::resolved_promise(s, v) {
        r.set(p.into())
    }
}
fn valid(s: &mut v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> bool {
    if record(s, o).is_some() {
        true
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        false
    }
}
fn uint(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
    n: u32,
) {
    if valid(s, a.this()) {
        r.set(v8::Integer::new_from_unsigned(s, n).into())
    }
}

fn profile_uint(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
    select: impl FnOnce(&crate::UsbDeviceFingerprint) -> u32,
) {
    if let Some(record) = record(s, a.this()) {
        r.set(v8::Integer::new_from_unsigned(s, select(&record.profile)).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}

fn usb_version_major(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    profile_uint(s, a, r, |profile| profile.usb_version_major as u32)
}
fn usb_version_minor(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    profile_uint(s, a, r, |profile| profile.usb_version_minor as u32)
}
fn usb_version_subminor(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    profile_uint(s, a, r, |profile| profile.usb_version_subminor as u32)
}
fn device_class(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    profile_uint(s, a, r, |profile| profile.device_class as u32)
}
fn device_subclass(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    profile_uint(s, a, r, |profile| profile.device_subclass as u32)
}
fn device_protocol(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    profile_uint(s, a, r, |profile| profile.device_protocol as u32)
}
fn vendor_id(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    profile_uint(s, a, r, |profile| profile.vendor_id as u32)
}
fn product_id(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    profile_uint(s, a, r, |profile| profile.product_id as u32)
}
fn device_version_major(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    profile_uint(s, a, r, |profile| profile.device_version_major as u32)
}
fn device_version_minor(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    profile_uint(s, a, r, |profile| profile.device_version_minor as u32)
}
fn device_version_subminor(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    profile_uint(s, a, r, |profile| profile.device_version_subminor as u32)
}
fn text(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
    f: impl FnOnce(DeviceData) -> Option<String>,
) {
    if let Some(v) = record(s, a.this()) {
        if let Some(value) = f(v).and_then(|value| v8::String::new(s, &value)) {
            r.set(value.into())
        } else {
            r.set(v8::null(s).into())
        }
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn manufacturer(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    text(s, a, r, |v| v.profile.manufacturer_name)
}
fn product(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    text(s, a, r, |v| v.profile.product_name)
}
fn serial(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    text(s, a, r, |v| v.profile.serial_number)
}
fn configuration(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Local::new(s, &v.configuration).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn configurations(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Local::new(s, &v.configurations).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn opened(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Boolean::new(s, v.opened).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn set_open(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
    value: bool,
) {
    if let Some(v) = s
        .get_slot_mut::<UsbDeviceStore>()
        .and_then(|x| x.records.get_mut(&a.this().get_identity_hash().get()))
    {
        v.opened = value;
        let x = v8::undefined(s);
        promise(s, x.into(), r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn open(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    set_open(s, a, r, true)
}
fn close(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    set_open(s, a, r, false)
}
fn void_method(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if valid(s, a.this()) {
        let x = v8::undefined(s);
        promise(s, x.into(), r)
    }
}
fn claim_interface(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let n = a.get(0).uint32_value(s).unwrap_or(0);
    super::usb_interface::set_claimed(s, n, true);
    void_method(s, a, r)
}
fn release_interface(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let n = a.get(0).uint32_value(s).unwrap_or(0);
    super::usb_interface::set_claimed(s, n, false);
    void_method(s, a, r)
}
fn control_in(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if !valid(s, a.this()) {
        return;
    }
    let length = a.get(1).uint32_value(s).unwrap_or(0) as usize;
    match super::usb_in_transfer_result::create(s, vec![0; length], "ok".to_owned()) {
        Ok(v) => promise(s, v.into(), r),
        Err(e) => crate::webidl::throw_type_error(s, &e),
    }
}
fn control_out(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if !valid(s, a.this()) {
        return;
    }
    match super::usb_out_transfer_result::create(s, 0, "ok".to_owned()) {
        Ok(v) => promise(s, v.into(), r),
        Err(e) => crate::webidl::throw_type_error(s, &e),
    }
}
fn iso_in(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if !valid(s, a.this()) {
        return;
    }
    let packet =
        match super::usb_isochronous_in_transfer_packet::create(s, "ok".to_owned(), Vec::new()) {
            Ok(v) => v,
            Err(e) => {
                crate::webidl::throw_type_error(s, &e);
                return;
            }
        };
    match super::usb_isochronous_in_transfer_result::create(s, Vec::new(), packet) {
        Ok(v) => promise(s, v.into(), r),
        Err(e) => crate::webidl::throw_type_error(s, &e),
    }
}
fn iso_out(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if !valid(s, a.this()) {
        return;
    }
    let packet = match super::usb_isochronous_out_transfer_packet::create(s, 0, "ok".to_owned()) {
        Ok(v) => v,
        Err(e) => {
            crate::webidl::throw_type_error(s, &e);
            return;
        }
    };
    match super::usb_isochronous_out_transfer_result::create(s, packet) {
        Ok(v) => promise(s, v.into(), r),
        Err(e) => crate::webidl::throw_type_error(s, &e),
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<UsbDeviceStore>() {
        store.constructor.remove(realm_id);
    }
}
