#![cfg(target_os = "windows")]

use crate::{
    EdgeRuntime, EdgeRuntimeOptions, Evaluation, FontBinarySourceFingerprint, LocalFontFingerprint,
};

fn text(runtime: &mut EdgeRuntime, source: &str) -> String {
    match runtime.evaluate(source).expect("JavaScript evaluation") {
        Evaluation::Undefined => "undefined".to_owned(),
        Evaluation::Null => "null".to_owned(),
        Evaluation::Boolean(value) => value.to_string(),
        Evaluation::Number(value) | Evaluation::String(value) | Evaluation::Other(value) => value,
    }
}

fn arial_options() -> EdgeRuntimeOptions {
    let path = r"C:\Windows\Fonts\arial.ttf";
    assert!(
        std::path::Path::new(path).is_file(),
        "Windows Arial is unavailable"
    );
    let mut options = EdgeRuntimeOptions::default();
    options.fingerprint.fonts.binary_sources = vec![FontBinarySourceFingerprint {
        family: "Binary Arial".to_owned(),
        path: path.to_owned(),
        face_index: 0,
    }];
    options
        .fingerprint
        .navigator
        .user_activation_has_been_active = true;
    options.fingerprint.navigator.user_activation_is_active = true;
    options
}

#[test]
fn configured_binary_font_drives_canvas_empty_space_kerning_and_ink_metrics() {
    let mut runtime = EdgeRuntime::with_options(arial_options()).expect("binary font runtime");
    assert_eq!(
        text(
            &mut runtime,
            r#"
            (() => {
              const context = document.createElement("canvas").getContext("2d");
              context.font = '13.3333px "Binary Arial"';
              return ["", "Hg", "AV", "A large contentful text"].map(value => {
                const metrics = context.measureText(value);
                return [
                  metrics.width,
                  metrics.actualBoundingBoxLeft,
                  metrics.actualBoundingBoxRight,
                  metrics.fontBoundingBoxAscent,
                  metrics.fontBoundingBoxDescent,
                  metrics.actualBoundingBoxAscent,
                  metrics.actualBoundingBoxDescent
                ];
              }).flat().join("|");
            })()
            "#,
        ),
        concat!(
            "0|0|0|12|3|0|0|",
            "17.039993286132812|0|16.626495361328125|12|3|10|3|",
            "16.792648315429688|1|16.901657104492188|12|3|10|0|",
            "128.93234252929688|1|129.22885131835938|12|3|10|3"
        )
    );
}

#[test]
fn canvas_font_setter_matches_edge_150_validation_and_serialization() {
    let mut runtime = EdgeRuntime::with_options(arial_options()).expect("binary font runtime");
    assert_eq!(
        text(
            &mut runtime,
            r#"
            (() => {
              const context = document.createElement("canvas").getContext("2d");
              const values = [
                "", "foo", "16px", "16px Arial", "bold 16px Arial",
                "16px Arial trailing", '16px "unterminated',
                "normal normal normal 16px Arial", "italic 700 16px/2 Arial",
                "16PX Arial", "0px Arial", "-1px Arial", "calc(16px) Arial",
                "16px var(--x)"
              ];
              return values.map(value => {
                context.font = value;
                return context.font;
              }).join("|");
            })()
            "#,
        ),
        concat!(
            "10px sans-serif|10px sans-serif|10px sans-serif|16px Arial|bold 16px Arial|",
            "16px \"Arial trailing\"|16px unterminated|16px Arial|italic bold 16px Arial|",
            "16px Arial|0px Arial|0px Arial|16px Arial|16px Arial"
        )
    );
}

#[test]
fn canvas_font_kerning_and_capital_features_drive_binary_shaping() {
    let mut runtime = EdgeRuntime::with_options(arial_options()).expect("binary font runtime");
    assert_eq!(
        text(
            &mut runtime,
            r#"
            (() => {
              const context = document.createElement("canvas").getContext("2d");
              context.font = '48px "Binary Arial"';
              const values = [];
              for (const kerning of ["auto", "normal", "none"]) {
                context.fontKerning = kerning;
                values.push(context.measureText("AV").width);
              }
              for (const caps of [
                "normal", "small-caps", "all-small-caps", "petite-caps",
                "all-petite-caps", "unicase", "titling-caps"
              ]) {
                context.fontVariantCaps = caps;
                values.push(context.measureText("Abc").width);
              }
              context.fontVariantCaps = "bad";
              context.fontStretch = "condensed";
              context.fontStretch = "75%";
              values.push(context.fontVariantCaps, context.fontStretch);
              context.fontStretch = "normal";
              context.fontVariantCaps = "normal";
              context.font = 'condensed 16px "Binary Arial"';
              values.push(context.font, context.fontStretch, context.fontVariantCaps);
              context.font = 'small-caps 16px "Binary Arial"';
              values.push(context.font, context.fontStretch, context.fontVariantCaps);
              return values.join("|");
            })()
            "#,
        ),
        concat!(
            "60.46875|60.46875|64.03125|",
            "82.7109375|87.796875|82.4296875|87.796875|82.4296875|",
            "101.296875|82.7109375|titling-caps|condensed|",
            "16px \"Binary Arial\"|condensed|normal|",
            "small-caps 16px \"Binary Arial\"|normal|small-caps"
        )
    );
}

#[test]
fn local_font_blob_is_real_octet_stream_and_dynamic_face_is_realm_scoped() {
    let mut options = arial_options();
    options.fingerprint.permissions.local_fonts = "granted".to_owned();
    options
        .fingerprint
        .navigator
        .user_activation_has_been_active = true;
    options.fingerprint.navigator.user_activation_is_active = true;
    options.fingerprint.fonts.local_fonts = vec![LocalFontFingerprint {
        postscript_name: "BinaryArial-Regular".to_owned(),
        full_name: "Binary Arial Regular".to_owned(),
        family: "Binary Arial".to_owned(),
        style: "Regular".to_owned(),
    }];
    let expected_size = std::fs::metadata(r"C:\Windows\Fonts\arial.ttf")
        .expect("Arial metadata")
        .len();
    let mut runtime = EdgeRuntime::with_options(options).expect("local font runtime");
    text(
        &mut runtime,
        r#"
        globalThis.fontAnswer = "pending";
        (async () => {
          const [metadata] = await queryLocalFonts({postscriptNames:["BinaryArial-Regular"]});
          const blob = await metadata.blob();
          const face = new FontFace("Dynamic Alias", await blob.arrayBuffer());
          await face.load();
          const frame = document.createElement("iframe");
          document.body.appendChild(frame);
          const main = document.createElement("canvas").getContext("2d");
          const child = frame.contentDocument.createElement("canvas").getContext("2d");
          main.font = child.font = '16px "Dynamic Alias"';
          const before = [main.measureText("Hg").width, child.measureText("Hg").width];
          frame.contentDocument.fonts.add(face);
          const during = [main.measureText("Hg").width, child.measureText("Hg").width];
          frame.contentDocument.fonts.delete(face);
          const after = [main.measureText("Hg").width, child.measureText("Hg").width];
          fontAnswer = [
            metadata.postscriptName, blob.type, blob.size,
            face.status, before, during, after
          ].join("|");
        })();
        "#,
    );
    assert_eq!(
        text(&mut runtime, "fontAnswer"),
        format!(
            "BinaryArial-Regular|application/octet-stream|{expected_size}|loaded|18.272000000000002,18.272000000000002|18.272000000000002,20.453125|18.272000000000002,18.272000000000002"
        )
    );
}

#[test]
fn local_font_query_without_transient_activation_rejects_security_error() {
    let mut options = EdgeRuntimeOptions::default();
    options.fingerprint.permissions.local_fonts = "granted".to_owned();
    let mut runtime = EdgeRuntime::with_options(options).expect("inactive font runtime");
    text(
        &mut runtime,
        r#"
        globalThis.activationAnswer = "pending";
        queryLocalFonts().then(
          fonts => `resolved:${fonts.length}`,
          error => `${error.name}:${error.message}`
        ).then(value => activationAnswer = value);
        "#,
    );
    assert_eq!(
        text(&mut runtime, "activationAnswer"),
        "SecurityError:User activation is required."
    );
}

#[test]
fn missing_local_font_bytes_reject_and_invalid_font_data_stays_error() {
    let mut options = EdgeRuntimeOptions::default();
    options.fingerprint.permissions.local_fonts = "granted".to_owned();
    options
        .fingerprint
        .navigator
        .user_activation_has_been_active = true;
    options.fingerprint.navigator.user_activation_is_active = true;
    options.fingerprint.fonts.local_fonts = vec![LocalFontFingerprint {
        postscript_name: "Unavailable-Regular".to_owned(),
        full_name: "Unavailable Regular".to_owned(),
        family: "Unavailable".to_owned(),
        style: "Regular".to_owned(),
    }];
    let mut runtime = EdgeRuntime::with_options(options).expect("missing font runtime");
    text(
        &mut runtime,
        r#"
        globalThis.failureAnswer = "pending";
        (async () => {
          const [metadata] = await queryLocalFonts();
          const blobFailure = await metadata.blob().then(
            () => "resolved", error => `${error.name}:${error.message}`
          );
          const invalid = new FontFace("Invalid", new Uint8Array([1, 2, 3]));
          const loadedFailure = await invalid.loaded.then(
            () => "resolved", error => error.name
          );
          failureAnswer = [blobFailure, invalid.status, loadedFailure].join("|");
        })();
        "#,
    );
    assert_eq!(
        text(&mut runtime, "failureAnswer"),
        concat!(
            "TypeError:Font data for Unavailable-Regular could not be accessed.|",
            "error|SyntaxError"
        )
    );
}

#[test]
fn native_trace_keeps_binary_font_values_identity_and_native_shape_unchanged() {
    let mut runtime = EdgeRuntime::with_options(arial_options()).expect("binary font runtime");
    let probe = r#"
        (() => {
          const context = document.createElement("canvas").getContext("2d");
          context.font = '16px "Binary Arial"';
          const metrics = context.measureText("Hg");
          return [
            metrics.width,
            metrics.actualBoundingBoxRight,
            context.measureText === CanvasRenderingContext2D.prototype.measureText,
            Function.prototype.toString.call(context.measureText)
          ].join("|");
        })()
    "#;
    let direct = text(&mut runtime, probe);
    runtime.enable_native_trace().expect("enable native trace");
    let traced = text(&mut runtime, probe);
    assert_eq!(traced, direct);
    assert_eq!(
        traced,
        "20.453125|20.5546875|true|function measureText() { [native code] }"
    );
    assert!(
        runtime
            .native_trace()
            .iter()
            .any(|entry| entry.api.ends_with(".measureText"))
    );
}
