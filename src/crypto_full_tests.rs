use crate::{EdgeRuntime, Evaluation};

fn text(runtime: &mut EdgeRuntime, source: &str) -> String {
    match runtime.evaluate(source).expect("JavaScript evaluation") {
        Evaluation::Undefined => "undefined".to_owned(),
        Evaluation::Null => "null".to_owned(),
        Evaluation::Boolean(value) => value.to_string(),
        Evaluation::Number(value) | Evaluation::String(value) | Evaluation::Other(value) => value,
    }
}

#[test]
fn digest_and_hmac_match_standard_vectors_and_key_shapes() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    text(
        &mut runtime,
        r#"
        globalThis.cryptoDigestAnswer = "pending";
        (async () => {
          const hex = value => Array.from(new Uint8Array(value), byte =>
            byte.toString(16).padStart(2, "0")
          ).join("");
          const abc = new TextEncoder().encode("abc");
          const sha1 = await crypto.subtle.digest("SHA-1", abc);
          const sha256 = await crypto.subtle.digest({ name: "sha-256" }, abc);
          const sha384 = await crypto.subtle.digest("SHA-384", abc);
          const sha512 = await crypto.subtle.digest("SHA-512", abc);
          const hmacMaterial = new Uint8Array(20).fill(0x0b);
          const hmacKey = await crypto.subtle.importKey(
            "raw",
            hmacMaterial,
            { name: "HMAC", hash: "SHA-256" },
            true,
            ["sign", "verify"]
          );
          const signature = await crypto.subtle.sign(
            "HMAC",
            hmacKey,
            new TextEncoder().encode("Hi There")
          );
          const verified = await crypto.subtle.verify(
            "HMAC",
            hmacKey,
            signature,
            new TextEncoder().encode("Hi There")
          );
          const exported = await crypto.subtle.exportKey("raw", hmacKey);
          cryptoDigestAnswer = [
            hex(sha1),
            hex(sha256),
            hex(sha384),
            hex(sha512),
            hex(signature),
            verified,
            hmacKey instanceof CryptoKey,
            Object.prototype.toString.call(hmacKey),
            hmacKey.type,
            hmacKey.extractable,
            hmacKey.algorithm.name,
            hmacKey.algorithm.hash.name,
            hmacKey.algorithm.length,
            hmacKey.usages.join(","),
            Object.isFrozen(hmacKey.usages),
            hex(exported)
          ].join("|");
        })().catch(error => cryptoDigestAnswer =
          "ERR:" + error.name + ":" + error.message
        );
        "#,
    );
    assert_eq!(
        text(&mut runtime, "cryptoDigestAnswer"),
        concat!(
            "a9993e364706816aba3e25717850c26c9cd0d89d|",
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad|",
            "cb00753f45a35e8bb5a03d699ac65007272c32ab0eded1631a8b605a43ff5bed",
            "8086072ba1e7cc2358baeca134c825a7|",
            "ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a",
            "2192992a274fc1a836ba3c23a3feebbd454d4423643ce80e2a9ac94fa54ca49f|",
            "b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7|",
            "true|true|[object CryptoKey]|secret|true|HMAC|SHA-256|160|",
            "sign,verify|true|",
            "0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b"
        )
    );
}

#[test]
fn aes_gcm_ctr_cbc_and_kw_match_vectors_or_round_trip() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    text(
        &mut runtime,
        r#"
        globalThis.cryptoAesAnswer = "pending";
        (async () => {
          const fromHex = hex => new Uint8Array(
            hex.match(/../g).map(byte => parseInt(byte, 16))
          );
          const hex = value => Array.from(new Uint8Array(value), byte =>
            byte.toString(16).padStart(2, "0")
          ).join("");

          const zeroKey = await crypto.subtle.importKey(
            "raw", new Uint8Array(16), "AES-GCM", false,
            ["encrypt", "decrypt"]
          );
          const gcm = await crypto.subtle.encrypt(
            { name: "AES-GCM", iv: new Uint8Array(12) },
            zeroKey,
            new Uint8Array(16)
          );
          const gcmPlain = await crypto.subtle.decrypt(
            { name: "AES-GCM", iv: new Uint8Array(12) },
            zeroKey,
            gcm
          );

          const ctrKey = await crypto.subtle.importKey(
            "raw",
            fromHex("2b7e151628aed2a6abf7158809cf4f3c"),
            "AES-CTR",
            false,
            ["encrypt", "decrypt"]
          );
          const ctr = await crypto.subtle.encrypt(
            {
              name: "AES-CTR",
              counter: fromHex("f0f1f2f3f4f5f6f7f8f9fafbfcfdfeff"),
              length: 128
            },
            ctrKey,
            fromHex("6bc1bee22e409f96e93d7e117393172a")
          );

          const cbcKey = await crypto.subtle.importKey(
            "raw",
            fromHex("2b7e151628aed2a6abf7158809cf4f3c"),
            "AES-CBC",
            false,
            ["encrypt", "decrypt"]
          );
          const cbcInput = fromHex("6bc1bee22e409f96e93d7e117393172a");
          const cbcIv = fromHex("000102030405060708090a0b0c0d0e0f");
          const cbc = await crypto.subtle.encrypt(
            { name: "AES-CBC", iv: cbcIv },
            cbcKey,
            cbcInput
          );
          const cbcPlain = await crypto.subtle.decrypt(
            { name: "AES-CBC", iv: cbcIv },
            cbcKey,
            cbc
          );

          const wrappingKey = await crypto.subtle.importKey(
            "raw",
            fromHex("000102030405060708090a0b0c0d0e0f"),
            "AES-KW",
            false,
            ["wrapKey", "unwrapKey"]
          );
          const wrappedKey = await crypto.subtle.importKey(
            "raw",
            fromHex("00112233445566778899aabbccddeeff"),
            "AES-GCM",
            true,
            ["encrypt"]
          );
          const wrapped = await crypto.subtle.wrapKey(
            "raw", wrappedKey, wrappingKey, "AES-KW"
          );
          const unwrapped = await crypto.subtle.unwrapKey(
            "raw",
            wrapped,
            wrappingKey,
            "AES-KW",
            "AES-GCM",
            true,
            ["encrypt"]
          );
          const unwrappedRaw = await crypto.subtle.exportKey("raw", unwrapped);

          cryptoAesAnswer = [
            hex(gcm),
            hex(gcmPlain),
            hex(ctr),
            hex(cbc).slice(0, 32),
            hex(cbcPlain),
            hex(wrapped),
            hex(unwrappedRaw),
            unwrapped.algorithm.name,
            unwrapped.algorithm.length
          ].join("|");
        })().catch(error => cryptoAesAnswer =
          "ERR:" + error.name + ":" + error.message
        );
        "#,
    );
    assert_eq!(
        text(&mut runtime, "cryptoAesAnswer"),
        concat!(
            "0388dace60b6a392f328c2b971b2fe78ab6e47d42cec13bdf53a67b21257bddf|",
            "00000000000000000000000000000000|",
            "874d6191b620e3261bef6864990db6ce|",
            "7649abac8119b246cee98e9b12e9197d|",
            "6bc1bee22e409f96e93d7e117393172a|",
            "1fa68b0a8112b447aef34bd8fb5a7b829d3e862371d2cfe5|",
            "00112233445566778899aabbccddeeff|AES-GCM|128"
        )
    );
}

#[test]
fn pbkdf2_hkdf_and_derive_key_match_standard_vectors() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    text(
        &mut runtime,
        r#"
        globalThis.cryptoKdfAnswer = "pending";
        (async () => {
          const hex = value => Array.from(new Uint8Array(value), byte =>
            byte.toString(16).padStart(2, "0")
          ).join("");
          const fromHex = value => new Uint8Array(
            value.match(/../g).map(byte => parseInt(byte, 16))
          );
          const encoder = new TextEncoder();
          const pbkdfKey = await crypto.subtle.importKey(
            "raw", encoder.encode("password"), "PBKDF2", false,
            ["deriveBits", "deriveKey"]
          );
          const pbkdfBits = await crypto.subtle.deriveBits(
            {
              name: "PBKDF2",
              hash: "SHA-256",
              salt: encoder.encode("salt"),
              iterations: 1
            },
            pbkdfKey,
            256
          );
          const derivedAes = await crypto.subtle.deriveKey(
            {
              name: "PBKDF2",
              hash: "SHA-256",
              salt: encoder.encode("salt"),
              iterations: 1
            },
            pbkdfKey,
            { name: "AES-GCM", length: 128 },
            true,
            ["encrypt"]
          );
          const derivedRaw = await crypto.subtle.exportKey("raw", derivedAes);

          const hkdfKey = await crypto.subtle.importKey(
            "raw",
            new Uint8Array(22).fill(0x0b),
            "HKDF",
            false,
            ["deriveBits"]
          );
          const hkdfBits = await crypto.subtle.deriveBits(
            {
              name: "HKDF",
              hash: "SHA-256",
              salt: fromHex("000102030405060708090a0b0c"),
              info: fromHex("f0f1f2f3f4f5f6f7f8f9")
            },
            hkdfKey,
            336
          );
          cryptoKdfAnswer = [
            hex(pbkdfBits),
            hex(derivedRaw),
            derivedAes.algorithm.name,
            derivedAes.algorithm.length,
            derivedAes.usages.join(","),
            hex(hkdfBits)
          ].join("|");
        })().catch(error => cryptoKdfAnswer =
          "ERR:" + error.name + ":" + error.message
        );
        "#,
    );
    assert_eq!(
        text(&mut runtime, "cryptoKdfAnswer"),
        concat!(
            "120fb6cffcf8b32c43e7225256c4f837a86548c92ccc35480805987cb70be17b|",
            "120fb6cffcf8b32c43e7225256c4f837|AES-GCM|128|encrypt|",
            "3cb25f25faacd57a90434f64d0362f2a",
            "2d2d0a90cf1a5a4c5db02d56ecc4c5bf",
            "34007208d5b887185865"
        )
    );
}

#[test]
fn subtle_crypto_rejects_wrong_usages_algorithms_and_authentication() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    text(
        &mut runtime,
        r#"
        globalThis.cryptoErrorAnswer = "pending";
        (async () => {
          const safe = async operation => {
            try {
              await operation();
              return "none";
            } catch (error) {
              return error.name;
            }
          };
          const data = new Uint8Array([1, 2, 3]);
          const key = await crypto.subtle.importKey(
            "raw", new Uint8Array(16), "AES-GCM", false,
            ["encrypt", "decrypt"]
          );
          const encrypted = await crypto.subtle.encrypt(
            { name: "AES-GCM", iv: new Uint8Array(12) }, key, data
          );
          new Uint8Array(encrypted)[0] ^= 1;
          cryptoErrorAnswer = [
            await safe(() => crypto.subtle.digest("NOPE", data)),
            await safe(() => crypto.subtle.importKey(
              "raw", new Uint8Array(16), "AES-GCM", false, ["sign"]
            )),
            await safe(() => crypto.subtle.decrypt(
              { name: "AES-GCM", iv: new Uint8Array(12) }, key, encrypted
            )),
            await safe(() => crypto.subtle.exportKey("raw", key))
          ].join("|");
        })().catch(error => cryptoErrorAnswer =
          "ERR:" + error.name + ":" + error.message
        );
        "#,
    );
    assert_eq!(
        text(&mut runtime, "cryptoErrorAnswer"),
        "NotSupportedError|SyntaxError|OperationError|InvalidAccessError"
    );
}
