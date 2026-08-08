use aes::cipher::{BlockDecrypt, BlockEncrypt, KeyInit};
use cipher::block_padding::Pkcs7;
use cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use hmac::{Hmac, Mac};
use sha1::Sha1;
use sha2::{Digest, Sha256, Sha384, Sha512};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum HashAlgorithm {
    Sha1,
    Sha256,
    Sha384,
    Sha512,
}

pub(crate) enum CipherParameters {
    AesCbc {
        iv: Vec<u8>,
    },
    AesCtr {
        counter: Vec<u8>,
        length: u32,
    },
    AesGcm {
        iv: Vec<u8>,
        additional_data: Vec<u8>,
        tag_length: u32,
    },
    AesKw,
}

impl HashAlgorithm {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Sha1 => "SHA-1",
            Self::Sha256 => "SHA-256",
            Self::Sha384 => "SHA-384",
            Self::Sha512 => "SHA-512",
        }
    }

    pub(crate) fn output_bits(self) -> u32 {
        match self {
            Self::Sha1 => 160,
            Self::Sha256 => 256,
            Self::Sha384 => 384,
            Self::Sha512 => 512,
        }
    }

    pub(crate) fn block_bytes(self) -> usize {
        match self {
            Self::Sha1 | Self::Sha256 => 64,
            Self::Sha384 | Self::Sha512 => 128,
        }
    }
}

pub(crate) fn require_receiver(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: &v8::FunctionCallbackArguments<'_>,
) -> bool {
    if super::subtle_crypto::valid(scope, arguments.this()) {
        true
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        false
    }
}

pub(crate) fn require_arguments(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: &v8::FunctionCallbackArguments<'_>,
    count: i32,
    operation: &str,
) -> bool {
    if arguments.length() >= count {
        true
    } else {
        crate::webidl::throw_type_error(
            scope,
            &format!(
                "Failed to execute '{operation}' on 'SubtleCrypto': {count} arguments required"
            ),
        );
        false
    }
}

pub(crate) fn algorithm_name(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> Result<String, String> {
    let name = if value.is_string() || value.is_string_object() {
        crate::webidl::value_to_string(scope, value)
    } else {
        let object = v8::Local::<v8::Object>::try_from(value)
            .map_err(|_| "Algorithm must be a string or an object".to_owned())?;
        let name = property(scope, object, "name")
            .ok_or_else(|| "Algorithm object is missing its name member".to_owned())?;
        crate::webidl::value_to_string(scope, name)
    };
    Ok(name.to_ascii_uppercase())
}

pub(crate) fn hash_algorithm(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> Result<HashAlgorithm, String> {
    match algorithm_name(scope, value)?.as_str() {
        "SHA-1" => Ok(HashAlgorithm::Sha1),
        "SHA-256" => Ok(HashAlgorithm::Sha256),
        "SHA-384" => Ok(HashAlgorithm::Sha384),
        "SHA-512" => Ok(HashAlgorithm::Sha512),
        name => Err(format!("Unrecognized hash algorithm '{name}'")),
    }
}

pub(crate) fn hash_property(
    scope: &mut v8::PinScope<'_, '_>,
    algorithm: v8::Local<'_, v8::Value>,
) -> Result<HashAlgorithm, String> {
    let object = v8::Local::<v8::Object>::try_from(algorithm)
        .map_err(|_| "Algorithm parameters must be an object".to_owned())?;
    let hash = property(scope, object, "hash")
        .ok_or_else(|| "Algorithm parameters are missing hash".to_owned())?;
    hash_algorithm(scope, hash)
}

pub(crate) fn property<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'s, v8::Object>,
    name: &str,
) -> Option<v8::Local<'s, v8::Value>> {
    let key = v8::String::new(scope, name)?;
    object.get(scope, key.into())
}

pub(crate) fn number_property(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Result<f64, String> {
    let value =
        property(scope, object, name).ok_or_else(|| format!("Algorithm is missing {name}"))?;
    value
        .number_value(scope)
        .ok_or_else(|| format!("Algorithm {name} is not a number"))
}

pub(crate) fn optional_number_property(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<f64> {
    property(scope, object, name)
        .filter(|value| !value.is_undefined())
        .and_then(|value| value.number_value(scope))
}

pub(crate) fn bytes(value: v8::Local<'_, v8::Value>) -> Option<Vec<u8>> {
    if let Ok(view) = v8::Local::<v8::ArrayBufferView>::try_from(value) {
        let mut output = vec![0; view.byte_length()];
        let copied = view.copy_contents(&mut output);
        output.truncate(copied);
        Some(output)
    } else if let Ok(buffer) = v8::Local::<v8::ArrayBuffer>::try_from(value) {
        let store = buffer.get_backing_store();
        let data = store.data()?;
        Some(
            unsafe { std::slice::from_raw_parts(data.as_ptr().cast::<u8>(), store.byte_length()) }
                .to_vec(),
        )
    } else {
        None
    }
}

pub(crate) fn bytes_property(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Result<Vec<u8>, String> {
    let value =
        property(scope, object, name).ok_or_else(|| format!("Algorithm is missing {name}"))?;
    bytes(value).ok_or_else(|| format!("Algorithm {name} must be a BufferSource"))
}

pub(crate) fn optional_bytes_property(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Result<Vec<u8>, String> {
    match property(scope, object, name) {
        Some(value) if !value.is_undefined() => {
            bytes(value).ok_or_else(|| format!("Algorithm {name} must be a BufferSource"))
        }
        _ => Ok(Vec::new()),
    }
}

pub(crate) fn usages(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> Result<Vec<String>, String> {
    let array = v8::Local::<v8::Array>::try_from(value)
        .map_err(|_| "keyUsages must be an Array".to_owned())?;
    let mut output = Vec::with_capacity(array.length() as usize);
    for index in 0..array.length() {
        let value = array
            .get_index(scope, index)
            .unwrap_or_else(|| v8::undefined(scope).into());
        let usage = crate::webidl::value_to_string(scope, value);
        if !matches!(
            usage.as_str(),
            "encrypt"
                | "decrypt"
                | "sign"
                | "verify"
                | "deriveKey"
                | "deriveBits"
                | "wrapKey"
                | "unwrapKey"
        ) {
            return Err(format!("'{usage}' is not a valid KeyUsage"));
        }
        if !output.contains(&usage) {
            output.push(usage);
        }
    }
    Ok(output)
}

pub(crate) fn validate_usages(usages: &[String], allowed: &[&str]) -> Result<(), String> {
    if let Some(usage) = usages
        .iter()
        .find(|usage| !allowed.contains(&usage.as_str()))
    {
        Err(format!(
            "The requested key usage '{usage}' is not valid for this algorithm"
        ))
    } else {
        Ok(())
    }
}

pub(crate) fn require_key_usage(
    key: &super::crypto_key::CryptoKeyRecord,
    usage: &str,
) -> Result<(), String> {
    if key.usages.iter().any(|candidate| candidate == usage) {
        Ok(())
    } else {
        Err(format!(
            "The CryptoKey does not support the '{usage}' operation"
        ))
    }
}

pub(crate) fn reject(
    scope: &mut v8::PinScope<'_, '_>,
    result: &mut v8::ReturnValue<'_>,
    name: &str,
    message: &str,
) {
    let Ok(exception) = super::dom_exception::create(scope, message.to_owned(), name.to_owned())
    else {
        return;
    };
    if let Ok(promise) = super::writable_stream::rejected_promise(scope, exception.into()) {
        result.set(promise.into());
    }
}

pub(crate) fn reject_not_supported(
    scope: &mut v8::PinScope<'_, '_>,
    result: &mut v8::ReturnValue<'_>,
    message: &str,
) {
    reject(scope, result, "NotSupportedError", message);
}

pub(crate) fn resolve_array_buffer(
    scope: &mut v8::PinScope<'_, '_>,
    result: &mut v8::ReturnValue<'_>,
    bytes: Vec<u8>,
) {
    let backing = v8::ArrayBuffer::new_backing_store_from_vec(bytes).make_shared();
    let buffer = v8::ArrayBuffer::with_backing_store(scope, &backing);
    resolve_value(scope, result, buffer.into());
}

pub(crate) fn resolve_bool(
    scope: &mut v8::PinScope<'_, '_>,
    result: &mut v8::ReturnValue<'_>,
    value: bool,
) {
    let value = v8::Boolean::new(scope, value);
    resolve_value(scope, result, value.into());
}

pub(crate) fn resolve_value(
    scope: &mut v8::PinScope<'_, '_>,
    result: &mut v8::ReturnValue<'_>,
    value: v8::Local<'_, v8::Value>,
) {
    if let Ok(promise) = super::writable_stream::resolved_promise(scope, value) {
        result.set(promise.into());
    }
}

pub(crate) fn digest(hash: HashAlgorithm, data: &[u8]) -> Vec<u8> {
    match hash {
        HashAlgorithm::Sha1 => Sha1::digest(data).to_vec(),
        HashAlgorithm::Sha256 => Sha256::digest(data).to_vec(),
        HashAlgorithm::Sha384 => Sha384::digest(data).to_vec(),
        HashAlgorithm::Sha512 => Sha512::digest(data).to_vec(),
    }
}

pub(crate) fn hmac(hash: HashAlgorithm, key: &[u8], data: &[u8]) -> Result<Vec<u8>, String> {
    match hash {
        HashAlgorithm::Sha1 => {
            let mut mac = <Hmac<Sha1> as Mac>::new_from_slice(key)
                .map_err(|_| "Invalid HMAC key".to_owned())?;
            mac.update(data);
            Ok(mac.finalize().into_bytes().to_vec())
        }
        HashAlgorithm::Sha256 => {
            let mut mac = <Hmac<Sha256> as Mac>::new_from_slice(key)
                .map_err(|_| "Invalid HMAC key".to_owned())?;
            mac.update(data);
            Ok(mac.finalize().into_bytes().to_vec())
        }
        HashAlgorithm::Sha384 => {
            let mut mac = <Hmac<Sha384> as Mac>::new_from_slice(key)
                .map_err(|_| "Invalid HMAC key".to_owned())?;
            mac.update(data);
            Ok(mac.finalize().into_bytes().to_vec())
        }
        HashAlgorithm::Sha512 => {
            let mut mac = <Hmac<Sha512> as Mac>::new_from_slice(key)
                .map_err(|_| "Invalid HMAC key".to_owned())?;
            mac.update(data);
            Ok(mac.finalize().into_bytes().to_vec())
        }
    }
}

pub(crate) fn constant_time_equal(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    left.iter()
        .zip(right)
        .fold(0_u8, |difference, (left, right)| {
            difference | (left ^ right)
        })
        == 0
}

pub(crate) fn cipher_parameters(
    scope: &mut v8::PinScope<'_, '_>,
    algorithm: v8::Local<'_, v8::Value>,
) -> Result<(String, CipherParameters), String> {
    let name = algorithm_name(scope, algorithm)?;
    let parameters = match name.as_str() {
        "AES-CBC" => {
            let object = v8::Local::<v8::Object>::try_from(algorithm)
                .map_err(|_| "AES-CBC parameters must be an object".to_owned())?;
            CipherParameters::AesCbc {
                iv: bytes_property(scope, object, "iv")?,
            }
        }
        "AES-CTR" => {
            let object = v8::Local::<v8::Object>::try_from(algorithm)
                .map_err(|_| "AES-CTR parameters must be an object".to_owned())?;
            let length = number_property(scope, object, "length")?;
            if !length.is_finite() || length.fract() != 0.0 {
                return Err("AES-CTR length must be an integer".to_owned());
            }
            CipherParameters::AesCtr {
                counter: bytes_property(scope, object, "counter")?,
                length: length as u32,
            }
        }
        "AES-GCM" => {
            let object = v8::Local::<v8::Object>::try_from(algorithm)
                .map_err(|_| "AES-GCM parameters must be an object".to_owned())?;
            let tag_length = optional_number_property(scope, object, "tagLength").unwrap_or(128.0);
            if !tag_length.is_finite() || tag_length.fract() != 0.0 {
                return Err("AES-GCM tagLength must be an integer".to_owned());
            }
            CipherParameters::AesGcm {
                iv: bytes_property(scope, object, "iv")?,
                additional_data: optional_bytes_property(scope, object, "additionalData")?,
                tag_length: tag_length as u32,
            }
        }
        "AES-KW" => CipherParameters::AesKw,
        _ => return Err(format!("The algorithm '{name}' is not a supported cipher")),
    };
    Ok((name, parameters))
}

pub(crate) fn encrypt_with_parameters(
    key: &[u8],
    parameters: &CipherParameters,
    plaintext: &[u8],
) -> Result<Vec<u8>, String> {
    match parameters {
        CipherParameters::AesCbc { iv } => aes_cbc_encrypt(key, iv, plaintext),
        CipherParameters::AesCtr { counter, length } => aes_ctr(key, counter, *length, plaintext),
        CipherParameters::AesGcm {
            iv,
            additional_data,
            tag_length,
        } => aes_gcm_encrypt(key, iv, additional_data, *tag_length, plaintext),
        CipherParameters::AesKw => aes_kw_wrap(key, plaintext),
    }
}

pub(crate) fn decrypt_with_parameters(
    key: &[u8],
    parameters: &CipherParameters,
    ciphertext: &[u8],
) -> Result<Vec<u8>, String> {
    match parameters {
        CipherParameters::AesCbc { iv } => aes_cbc_decrypt(key, iv, ciphertext),
        CipherParameters::AesCtr { counter, length } => aes_ctr(key, counter, *length, ciphertext),
        CipherParameters::AesGcm {
            iv,
            additional_data,
            tag_length,
        } => aes_gcm_decrypt(key, iv, additional_data, *tag_length, ciphertext),
        CipherParameters::AesKw => aes_kw_unwrap(key, ciphertext),
    }
}

pub(crate) fn derive_bits_material(
    scope: &mut v8::PinScope<'_, '_>,
    algorithm: v8::Local<'_, v8::Value>,
    base_key: &super::crypto_key::CryptoKeyRecord,
    length_bits: u32,
) -> Result<Vec<u8>, String> {
    if length_bits == 0 || length_bits % 8 != 0 {
        return Err("The requested derived bit length must be a positive multiple of 8".to_owned());
    }
    let output_length = (length_bits / 8) as usize;
    let object = v8::Local::<v8::Object>::try_from(algorithm)
        .map_err(|_| "Derivation algorithm parameters must be an object".to_owned())?;
    let name = algorithm_name(scope, algorithm)?;
    let mut output = vec![0_u8; output_length];
    match (name.as_str(), &base_key.algorithm) {
        ("PBKDF2", super::crypto_key::KeyAlgorithm::Pbkdf2) => {
            let salt = bytes_property(scope, object, "salt")?;
            let iterations = number_property(scope, object, "iterations")?;
            if !iterations.is_finite()
                || iterations <= 0.0
                || iterations.fract() != 0.0
                || iterations > u32::MAX as f64
            {
                return Err("PBKDF2 iterations must be a positive 32-bit integer".to_owned());
            }
            let hash_value = property(scope, object, "hash")
                .ok_or_else(|| "PBKDF2 parameters are missing hash".to_owned())?;
            let hash = hash_algorithm(scope, hash_value)?;
            match hash {
                HashAlgorithm::Sha1 => {
                    pbkdf2::pbkdf2_hmac::<Sha1>(
                        &base_key.material,
                        &salt,
                        iterations as u32,
                        &mut output,
                    );
                }
                HashAlgorithm::Sha256 => {
                    pbkdf2::pbkdf2_hmac::<Sha256>(
                        &base_key.material,
                        &salt,
                        iterations as u32,
                        &mut output,
                    );
                }
                HashAlgorithm::Sha384 => {
                    pbkdf2::pbkdf2_hmac::<Sha384>(
                        &base_key.material,
                        &salt,
                        iterations as u32,
                        &mut output,
                    );
                }
                HashAlgorithm::Sha512 => {
                    pbkdf2::pbkdf2_hmac::<Sha512>(
                        &base_key.material,
                        &salt,
                        iterations as u32,
                        &mut output,
                    );
                }
            }
        }
        ("HKDF", super::crypto_key::KeyAlgorithm::Hkdf) => {
            let salt = bytes_property(scope, object, "salt")?;
            let info = bytes_property(scope, object, "info")?;
            let hash_value = property(scope, object, "hash")
                .ok_or_else(|| "HKDF parameters are missing hash".to_owned())?;
            let hash = hash_algorithm(scope, hash_value)?;
            match hash {
                HashAlgorithm::Sha1 => hkdf::Hkdf::<Sha1>::new(Some(&salt), &base_key.material)
                    .expand(&info, &mut output)
                    .map_err(|_| "The requested HKDF output is too long".to_owned())?,
                HashAlgorithm::Sha256 => hkdf::Hkdf::<Sha256>::new(Some(&salt), &base_key.material)
                    .expand(&info, &mut output)
                    .map_err(|_| "The requested HKDF output is too long".to_owned())?,
                HashAlgorithm::Sha384 => hkdf::Hkdf::<Sha384>::new(Some(&salt), &base_key.material)
                    .expand(&info, &mut output)
                    .map_err(|_| "The requested HKDF output is too long".to_owned())?,
                HashAlgorithm::Sha512 => hkdf::Hkdf::<Sha512>::new(Some(&salt), &base_key.material)
                    .expand(&info, &mut output)
                    .map_err(|_| "The requested HKDF output is too long".to_owned())?,
            }
        }
        _ => {
            return Err(format!(
                "The derivation algorithm '{name}' does not match the base key"
            ));
        }
    }
    Ok(output)
}

pub(crate) fn aes_gcm_encrypt(
    key: &[u8],
    iv: &[u8],
    additional_data: &[u8],
    tag_length: u32,
    plaintext: &[u8],
) -> Result<Vec<u8>, String> {
    let tag_bytes = gcm_tag_bytes(tag_length)?;
    if iv.is_empty() {
        return Err("AES-GCM iv must not be empty".to_owned());
    }
    let hash_subkey = encrypt_aes_block(key, [0_u8; 16])?;
    let initial_counter = gcm_initial_counter(hash_subkey, iv);
    let output = gcm_counter_mode(key, initial_counter, plaintext)?;
    let authentication =
        gcm_authenticate(hash_subkey, initial_counter, additional_data, &output, key)?;
    let mut output_with_tag = output;
    output_with_tag.extend_from_slice(&authentication[..tag_bytes]);
    Ok(output_with_tag)
}

fn gcm_tag_bytes(tag_length: u32) -> Result<usize, String> {
    match tag_length {
        32 | 64 | 96 | 104 | 112 | 120 | 128 => Ok((tag_length / 8) as usize),
        _ => Err("AES-GCM tagLength must be 32, 64, 96, 104, 112, 120, or 128".to_owned()),
    }
}

fn gcm_initial_counter(hash_subkey: [u8; 16], iv: &[u8]) -> [u8; 16] {
    if iv.len() == 12 {
        let mut counter = [0_u8; 16];
        counter[..12].copy_from_slice(iv);
        counter[15] = 1;
        return counter;
    }
    let mut input = iv.to_vec();
    while input.len() % 16 != 8 {
        input.push(0);
    }
    input.extend_from_slice(&((iv.len() as u64).saturating_mul(8)).to_be_bytes());
    ghash(hash_subkey, &input)
}

fn gcm_counter_mode(
    key: &[u8],
    initial_counter: [u8; 16],
    input: &[u8],
) -> Result<Vec<u8>, String> {
    let mut counter = initial_counter;
    increment_gcm_counter(&mut counter);
    let mut output = Vec::with_capacity(input.len());
    for chunk in input.chunks(16) {
        let stream = encrypt_aes_block(key, counter)?;
        output.extend(chunk.iter().zip(stream).map(|(byte, mask)| byte ^ mask));
        increment_gcm_counter(&mut counter);
    }
    Ok(output)
}

fn gcm_authenticate(
    hash_subkey: [u8; 16],
    initial_counter: [u8; 16],
    additional_data: &[u8],
    ciphertext: &[u8],
    key: &[u8],
) -> Result<[u8; 16], String> {
    let mut input = additional_data.to_vec();
    while input.len() % 16 != 0 {
        input.push(0);
    }
    input.extend_from_slice(ciphertext);
    while input.len() % 16 != 0 {
        input.push(0);
    }
    input.extend_from_slice(&((additional_data.len() as u64).saturating_mul(8)).to_be_bytes());
    input.extend_from_slice(&((ciphertext.len() as u64).saturating_mul(8)).to_be_bytes());
    let authentication = ghash(hash_subkey, &input);
    let encrypted_counter = encrypt_aes_block(key, initial_counter)?;
    Ok(std::array::from_fn(|index| {
        authentication[index] ^ encrypted_counter[index]
    }))
}

pub(crate) fn aes_gcm_decrypt(
    key: &[u8],
    iv: &[u8],
    additional_data: &[u8],
    tag_length: u32,
    ciphertext_and_tag: &[u8],
) -> Result<Vec<u8>, String> {
    let tag_bytes = gcm_tag_bytes(tag_length)?;
    if iv.is_empty() || ciphertext_and_tag.len() < tag_bytes {
        return Err("Invalid AES-GCM parameters or ciphertext".to_owned());
    }
    let split = ciphertext_and_tag.len() - tag_bytes;
    let ciphertext = &ciphertext_and_tag[..split];
    let supplied_tag = &ciphertext_and_tag[split..];
    let hash_subkey = encrypt_aes_block(key, [0_u8; 16])?;
    let initial_counter = gcm_initial_counter(hash_subkey, iv);
    let expected_tag = gcm_authenticate(
        hash_subkey,
        initial_counter,
        additional_data,
        ciphertext,
        key,
    )?;
    if !constant_time_equal(supplied_tag, &expected_tag[..tag_bytes]) {
        return Err("AES-GCM authentication failed".to_owned());
    }
    gcm_counter_mode(key, initial_counter, ciphertext)
}

fn ghash(hash_subkey: [u8; 16], input: &[u8]) -> [u8; 16] {
    debug_assert_eq!(input.len() % 16, 0);
    let hash_subkey = u128::from_be_bytes(hash_subkey);
    let mut state = 0_u128;
    for block in input.chunks_exact(16) {
        let block =
            u128::from_be_bytes(<[u8; 16]>::try_from(block).expect("GHASH block is 16 bytes"));
        state = ghash_multiply(state ^ block, hash_subkey);
    }
    state.to_be_bytes()
}

fn ghash_multiply(left: u128, right: u128) -> u128 {
    let mut output = 0_u128;
    let mut value = right;
    for bit in 0..128 {
        if left & (1_u128 << (127 - bit)) != 0 {
            output ^= value;
        }
        value = if value & 1 == 0 {
            value >> 1
        } else {
            (value >> 1) ^ 0xe100_0000_0000_0000_0000_0000_0000_0000_u128
        };
    }
    output
}

fn increment_gcm_counter(counter: &mut [u8; 16]) {
    let value = u32::from_be_bytes(counter[12..].try_into().expect("GCM counter suffix"));
    counter[12..].copy_from_slice(&value.wrapping_add(1).to_be_bytes());
}

pub(crate) fn aes_cbc_encrypt(key: &[u8], iv: &[u8], plaintext: &[u8]) -> Result<Vec<u8>, String> {
    if iv.len() != 16 {
        return Err("AES-CBC iv must be 16 bytes".to_owned());
    }
    match key.len() {
        16 => Ok(cbc::Encryptor::<aes::Aes128>::new_from_slices(key, iv)
            .map_err(|_| "Invalid AES-CBC key or iv".to_owned())?
            .encrypt_padded_vec_mut::<Pkcs7>(plaintext)),
        24 => Ok(cbc::Encryptor::<aes::Aes192>::new_from_slices(key, iv)
            .map_err(|_| "Invalid AES-CBC key or iv".to_owned())?
            .encrypt_padded_vec_mut::<Pkcs7>(plaintext)),
        32 => Ok(cbc::Encryptor::<aes::Aes256>::new_from_slices(key, iv)
            .map_err(|_| "Invalid AES-CBC key or iv".to_owned())?
            .encrypt_padded_vec_mut::<Pkcs7>(plaintext)),
        _ => Err("Invalid AES key length".to_owned()),
    }
}

pub(crate) fn aes_cbc_decrypt(key: &[u8], iv: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>, String> {
    if iv.len() != 16 || ciphertext.is_empty() || ciphertext.len() % 16 != 0 {
        return Err("Invalid AES-CBC iv or ciphertext length".to_owned());
    }
    match key.len() {
        16 => cbc::Decryptor::<aes::Aes128>::new_from_slices(key, iv)
            .map_err(|_| "Invalid AES-CBC key or iv".to_owned())?
            .decrypt_padded_vec_mut::<Pkcs7>(ciphertext)
            .map_err(|_| "AES-CBC padding is invalid".to_owned()),
        24 => cbc::Decryptor::<aes::Aes192>::new_from_slices(key, iv)
            .map_err(|_| "Invalid AES-CBC key or iv".to_owned())?
            .decrypt_padded_vec_mut::<Pkcs7>(ciphertext)
            .map_err(|_| "AES-CBC padding is invalid".to_owned()),
        32 => cbc::Decryptor::<aes::Aes256>::new_from_slices(key, iv)
            .map_err(|_| "Invalid AES-CBC key or iv".to_owned())?
            .decrypt_padded_vec_mut::<Pkcs7>(ciphertext)
            .map_err(|_| "AES-CBC padding is invalid".to_owned()),
        _ => Err("Invalid AES key length".to_owned()),
    }
}

pub(crate) fn aes_ctr(
    key: &[u8],
    counter: &[u8],
    counter_bits: u32,
    input: &[u8],
) -> Result<Vec<u8>, String> {
    if counter.len() != 16 || !(1..=128).contains(&counter_bits) {
        return Err("Invalid AES-CTR counter or length".to_owned());
    }
    let block_count = input.len().div_ceil(16);
    if counter_bits < usize::BITS && block_count > (1_usize << counter_bits) {
        return Err("AES-CTR counter would repeat".to_owned());
    }
    let mut counter =
        <[u8; 16]>::try_from(counter).map_err(|_| "AES-CTR counter must be 16 bytes".to_owned())?;
    let mut output = Vec::with_capacity(input.len());
    for chunk in input.chunks(16) {
        let stream = encrypt_aes_block(key, counter)?;
        output.extend(chunk.iter().zip(stream).map(|(byte, mask)| byte ^ mask));
        increment_rightmost_bits(&mut counter, counter_bits);
    }
    Ok(output)
}

pub(crate) fn aes_kw_wrap(key: &[u8], plaintext: &[u8]) -> Result<Vec<u8>, String> {
    if plaintext.len() < 16 || plaintext.len() % 8 != 0 {
        return Err("AES-KW input must contain at least two 64-bit blocks".to_owned());
    }
    let block_count = plaintext.len() / 8;
    let mut a = [0xa6_u8; 8];
    let mut blocks = plaintext
        .chunks_exact(8)
        .map(|chunk| <[u8; 8]>::try_from(chunk).expect("8-byte AES-KW block"))
        .collect::<Vec<_>>();
    for round in 0..6 {
        for (index, block) in blocks.iter_mut().enumerate() {
            let mut input = [0_u8; 16];
            input[..8].copy_from_slice(&a);
            input[8..].copy_from_slice(block);
            let encrypted = encrypt_aes_block(key, input)?;
            let t = (round * block_count + index + 1) as u64;
            a.copy_from_slice(&encrypted[..8]);
            xor_u64_be(&mut a, t);
            block.copy_from_slice(&encrypted[8..]);
        }
    }
    let mut output = Vec::with_capacity(plaintext.len() + 8);
    output.extend_from_slice(&a);
    for block in blocks {
        output.extend_from_slice(&block);
    }
    Ok(output)
}

pub(crate) fn aes_kw_unwrap(key: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>, String> {
    if ciphertext.len() < 24 || ciphertext.len() % 8 != 0 {
        return Err("AES-KW ciphertext has an invalid length".to_owned());
    }
    let block_count = ciphertext.len() / 8 - 1;
    let mut a = <[u8; 8]>::try_from(&ciphertext[..8]).expect("8-byte AES-KW register");
    let mut blocks = ciphertext[8..]
        .chunks_exact(8)
        .map(|chunk| <[u8; 8]>::try_from(chunk).expect("8-byte AES-KW block"))
        .collect::<Vec<_>>();
    for round in (0..6).rev() {
        for index in (0..block_count).rev() {
            let t = (round * block_count + index + 1) as u64;
            let mut input = [0_u8; 16];
            let mut register = a;
            xor_u64_be(&mut register, t);
            input[..8].copy_from_slice(&register);
            input[8..].copy_from_slice(&blocks[index]);
            let decrypted = decrypt_aes_block(key, input)?;
            a.copy_from_slice(&decrypted[..8]);
            blocks[index].copy_from_slice(&decrypted[8..]);
        }
    }
    if !constant_time_equal(&a, &[0xa6_u8; 8]) {
        return Err("AES-KW integrity check failed".to_owned());
    }
    Ok(blocks.into_iter().flatten().collect())
}

fn encrypt_aes_block(key: &[u8], block: [u8; 16]) -> Result<[u8; 16], String> {
    let mut block = aes::Block::clone_from_slice(&block);
    match key.len() {
        16 => aes::Aes128::new_from_slice(key)
            .map_err(|_| "Invalid AES key".to_owned())?
            .encrypt_block(&mut block),
        24 => aes::Aes192::new_from_slice(key)
            .map_err(|_| "Invalid AES key".to_owned())?
            .encrypt_block(&mut block),
        32 => aes::Aes256::new_from_slice(key)
            .map_err(|_| "Invalid AES key".to_owned())?
            .encrypt_block(&mut block),
        _ => return Err("Invalid AES key length".to_owned()),
    }
    Ok(block.into())
}

fn decrypt_aes_block(key: &[u8], block: [u8; 16]) -> Result<[u8; 16], String> {
    let mut block = aes::Block::clone_from_slice(&block);
    match key.len() {
        16 => aes::Aes128::new_from_slice(key)
            .map_err(|_| "Invalid AES key".to_owned())?
            .decrypt_block(&mut block),
        24 => aes::Aes192::new_from_slice(key)
            .map_err(|_| "Invalid AES key".to_owned())?
            .decrypt_block(&mut block),
        32 => aes::Aes256::new_from_slice(key)
            .map_err(|_| "Invalid AES key".to_owned())?
            .decrypt_block(&mut block),
        _ => return Err("Invalid AES key length".to_owned()),
    }
    Ok(block.into())
}

fn increment_rightmost_bits(counter: &mut [u8; 16], bit_length: u32) {
    for bit in 0..bit_length {
        let byte_index = 15 - (bit / 8) as usize;
        let mask = 1_u8 << (bit % 8);
        if counter[byte_index] & mask == 0 {
            counter[byte_index] |= mask;
            return;
        }
        counter[byte_index] &= !mask;
    }
}

fn xor_u64_be(bytes: &mut [u8; 8], value: u64) {
    for (target, mask) in bytes.iter_mut().zip(value.to_be_bytes()) {
        *target ^= mask;
    }
}
