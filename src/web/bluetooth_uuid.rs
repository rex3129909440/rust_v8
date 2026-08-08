#[derive(Clone, Copy)]
enum UuidKind {
    Service,
    Characteristic,
    Descriptor,
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = crate::webidl::create_function(
        scope,
        "BluetoothUUID",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::define_method(
        scope,
        constructor.into(),
        "canonicalUUID",
        1,
        canonical_uuid,
    )?;
    crate::webidl::define_method(
        scope,
        constructor.into(),
        "getCharacteristic",
        1,
        get_characteristic,
    )?;
    crate::webidl::define_method(
        scope,
        constructor.into(),
        "getDescriptor",
        1,
        get_descriptor,
    )?;
    crate::webidl::define_method(scope, constructor.into(), "getService", 1, get_service)?;
    crate::webidl::define_global(scope, "BluetoothUUID", constructor.into())
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'BluetoothUUID': Illegal constructor",
    );
}

fn canonical_uuid(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if arguments.length() < 1 || !arguments.get(0).is_number() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'canonicalUUID' on 'BluetoothUUID': Value is not of type 'unsigned long'.",
        );
        return;
    }
    let Some(alias) = arguments.get(0).uint32_value(scope) else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'canonicalUUID' on 'BluetoothUUID': Value is not of type 'unsigned long'.",
        );
        return;
    };
    return_uuid(scope, canonical(alias), result);
}

fn get_service(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    resolve(scope, arguments, result, UuidKind::Service, "getService");
}

fn get_characteristic(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    resolve(
        scope,
        arguments,
        result,
        UuidKind::Characteristic,
        "getCharacteristic",
    );
}

fn get_descriptor(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    resolve(
        scope,
        arguments,
        result,
        UuidKind::Descriptor,
        "getDescriptor",
    );
}

fn resolve(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
    kind: UuidKind,
    method: &str,
) {
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            &format!(
                "Failed to execute '{method}' on 'BluetoothUUID': 1 argument required, but only 0 present."
            ),
        );
        return;
    }
    let value = arguments.get(0);
    let uuid = if value.is_number() {
        value.uint32_value(scope).map(canonical)
    } else {
        let name = crate::webidl::value_to_string(scope, value).to_ascii_lowercase();
        if is_uuid(&name) {
            Some(name)
        } else {
            alias(kind, &name).map(canonical)
        }
    };
    let Some(uuid) = uuid else {
        let name = crate::webidl::value_to_string(scope, value);
        let noun = match kind {
            UuidKind::Service => "Service",
            UuidKind::Characteristic => "Characteristic",
            UuidKind::Descriptor => "Descriptor",
        };
        crate::webidl::throw_type_error(
            scope,
            &format!(
                "Failed to execute '{method}' on 'BluetoothUUID': Invalid {noun} name: '{name}'. It must be a valid UUID alias, UUID, or recognized standard name."
            ),
        );
        return;
    };
    return_uuid(scope, uuid, result);
}

fn canonical(alias: u32) -> String {
    format!("{alias:08x}-0000-1000-8000-00805f9b34fb")
}

fn is_uuid(value: &str) -> bool {
    value.len() == 36
        && value.as_bytes().get(8) == Some(&b'-')
        && value.as_bytes().get(13) == Some(&b'-')
        && value.as_bytes().get(18) == Some(&b'-')
        && value.as_bytes().get(23) == Some(&b'-')
        && value.chars().enumerate().all(|(index, character)| {
            matches!(index, 8 | 13 | 18 | 23) || character.is_ascii_hexdigit()
        })
}

fn alias(kind: UuidKind, name: &str) -> Option<u32> {
    match kind {
        UuidKind::Service => match name {
            "alert_notification" => Some(0x1811),
            "battery_service" => Some(0x180f),
            "blood_pressure" => Some(0x1810),
            "device_information" => Some(0x180a),
            "generic_access" => Some(0x1800),
            "generic_attribute" => Some(0x1801),
            "heart_rate" => Some(0x180d),
            "human_interface_device" => Some(0x1812),
            "immediate_alert" => Some(0x1802),
            "link_loss" => Some(0x1803),
            "running_speed_and_cadence" => Some(0x1814),
            "tx_power" => Some(0x1804),
            _ => None,
        },
        UuidKind::Characteristic => match name {
            "battery_level" => Some(0x2a19),
            "body_sensor_location" => Some(0x2a38),
            "device_name" => Some(0x2a00),
            "firmware_revision_string" => Some(0x2a26),
            "heart_rate_control_point" => Some(0x2a39),
            "heart_rate_measurement" => Some(0x2a37),
            "manufacturer_name_string" => Some(0x2a29),
            "model_number_string" => Some(0x2a24),
            "serial_number_string" => Some(0x2a25),
            "software_revision_string" => Some(0x2a28),
            _ => None,
        },
        UuidKind::Descriptor => match name {
            "gatt.characteristic_extended_properties" => Some(0x2900),
            "gatt.characteristic_user_description" => Some(0x2901),
            "gatt.client_characteristic_configuration" => Some(0x2902),
            "gatt.server_characteristic_configuration" => Some(0x2903),
            "gatt.characteristic_presentation_format" => Some(0x2904),
            "gatt.characteristic_aggregate_format" => Some(0x2905),
            "valid_range" => Some(0x2906),
            _ => None,
        },
    }
}

fn return_uuid(scope: &mut v8::PinScope<'_, '_>, uuid: String, mut result: v8::ReturnValue<'_>) {
    if let Some(value) = v8::String::new(scope, &uuid) {
        result.set(value.into());
    }
}
