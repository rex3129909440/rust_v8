use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, OnceLock};

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct ShapeMetrics {
    pub advance: f64,
    /// Distance from the alignment point to the left edge of the ink.
    pub actual_left: f64,
    /// Distance from the alignment point to the right edge of the ink.
    pub actual_right: f64,
    pub font_ascent: f64,
    pub font_descent: f64,
    pub actual_ascent: f64,
    pub actual_descent: f64,
    /// True when the metrics came from a host/system face. Chromium routes
    /// those faces through the platform rasterizer, which snaps ink bounds.
    pub platform_face: bool,
}

#[derive(Clone)]
struct LoadedFace {
    family: String,
    data: Arc<Vec<u8>>,
    face_index: u32,
    style: fontdb::Style,
    weight: fontdb::Weight,
    stretch: fontdb::Stretch,
    platform_face: bool,
    /// Native platform font object used for hinted/raster ink bounds.  Keep it
    /// with the loaded face: constructing DirectWrite/CoreText/FreeType state
    /// for every measureText()/Range query is both observably slow and unlike
    /// Chromium's per-font-cache lifetime.
    native_font: Option<Rc<font_kit::font::Font>>,
    /// Platform fallback APIs may request a face-specific scale (notably for
    /// CJK/emoji fallback on DirectWrite and CoreText).
    size_scale: f64,
}

#[derive(Clone)]
struct DynamicFace {
    realm_id: i32,
    identity: i32,
    face: LoadedFace,
    references: usize,
}

#[derive(Default)]
pub(crate) struct FontShapingState {
    explicit: Vec<LoadedFace>,
    dynamic: std::cell::RefCell<Vec<DynamicFace>>,
    system: Option<Arc<fontdb::Database>>,
    system_cache: std::cell::RefCell<HashMap<String, Option<LoadedFace>>>,
    fallback_cache: std::cell::RefCell<HashMap<String, Option<LoadedFace>>>,
}

static SYSTEM_FONT_DATABASE: OnceLock<Arc<fontdb::Database>> = OnceLock::new();

pub(crate) fn prepare(
    isolate: &mut v8::OwnedIsolate,
    profile: &crate::FontFingerprint,
) -> Result<(), String> {
    let mut state = FontShapingState::default();
    for source in &profile.binary_sources {
        let bytes = std::fs::read(&source.path).map_err(|error| {
            format!("cannot read font binary source '{}': {error}", source.path)
        })?;
        let face = rustybuzz::Face::from_slice(&bytes, source.face_index).ok_or_else(|| {
            format!(
                "font binary source '{}' face {} is not a supported OpenType face",
                source.path, source.face_index
            )
        })?;
        let (style, weight, stretch) = face_traits(face.as_ref());
        let data = Arc::new(bytes);
        state.explicit.push(LoadedFace {
            family: source.family.clone(),
            native_font: native_font(data.clone(), source.face_index),
            size_scale: 1.0,
            data,
            face_index: source.face_index,
            style,
            weight,
            stretch,
            platform_face: false,
        });
    }
    if profile.use_system_fonts {
        state.system = Some(
            SYSTEM_FONT_DATABASE
                .get_or_init(|| {
                    let mut database = fontdb::Database::new();
                    database.load_system_fonts();
                    Arc::new(database)
                })
                .clone(),
        );
    }
    isolate.set_slot(state);
    Ok(())
}

pub(crate) fn register_dynamic(
    scope: &v8::PinScope<'_, '_>,
    realm_id: i32,
    identity: i32,
    family: &str,
    style: &str,
    weight: &str,
    stretch: &str,
    bytes: Arc<Vec<u8>>,
) -> Result<(), String> {
    let face = loaded_dynamic_face(family, style, weight, stretch, bytes)?;
    let state = scope
        .get_slot::<FontShapingState>()
        .ok_or_else(|| "font shaping state was not prepared".to_owned())?;
    let mut dynamic = state.dynamic.borrow_mut();
    match dynamic
        .iter_mut()
        .find(|entry| entry.realm_id == realm_id && entry.identity == identity)
    {
        Some(current) => {
            current.references = current.references.saturating_add(1);
            current.face = face;
        }
        None => {
            dynamic.push(DynamicFace {
                realm_id,
                identity,
                face,
                references: 1,
            });
        }
    }
    Ok(())
}

pub(crate) fn refresh_dynamic(
    scope: &v8::PinScope<'_, '_>,
    identity: i32,
    family: &str,
    style: &str,
    weight: &str,
    stretch: &str,
    bytes: Arc<Vec<u8>>,
) -> Result<(), String> {
    let face = loaded_dynamic_face(family, style, weight, stretch, bytes)?;
    let Some(state) = scope.get_slot::<FontShapingState>() else {
        return Err("font shaping state was not prepared".to_owned());
    };
    for current in state
        .dynamic
        .borrow_mut()
        .iter_mut()
        .filter(|entry| entry.identity == identity)
    {
        current.face = face.clone();
    }
    Ok(())
}

pub(crate) fn unregister_dynamic(scope: &v8::PinScope<'_, '_>, realm_id: i32, identity: i32) {
    let Some(state) = scope.get_slot::<FontShapingState>() else {
        return;
    };
    let mut dynamic = state.dynamic.borrow_mut();
    if let Some(index) = dynamic
        .iter()
        .position(|entry| entry.realm_id == realm_id && entry.identity == identity)
    {
        let current = &mut dynamic[index];
        current.references = current.references.saturating_sub(1);
        if current.references == 0 {
            dynamic.remove(index);
        }
    }
}

pub(crate) fn cleanup_realm(scope: &v8::PinScope<'_, '_>, realm_id: i32) {
    let Some(state) = scope.get_slot::<FontShapingState>() else {
        return;
    };
    state
        .dynamic
        .borrow_mut()
        .retain(|entry| entry.realm_id != realm_id);
}

fn loaded_dynamic_face(
    family: &str,
    style: &str,
    weight: &str,
    stretch: &str,
    bytes: Arc<Vec<u8>>,
) -> Result<LoadedFace, String> {
    let parsed = rustybuzz::Face::from_slice(bytes.as_slice(), 0)
        .ok_or_else(|| "FontFace source is not a supported OpenType face".to_owned())?;
    let (native_style, native_weight, native_stretch) = face_traits(parsed.as_ref());
    Ok(LoadedFace {
        family: family.to_owned(),
        native_font: native_font(bytes.clone(), 0),
        size_scale: 1.0,
        data: bytes,
        face_index: 0,
        style: descriptor_style(style).unwrap_or(native_style),
        weight: descriptor_weight(weight).unwrap_or(native_weight),
        stretch: descriptor_stretch(stretch).unwrap_or(native_stretch),
        platform_face: false,
    })
}

pub(crate) fn local_font_bytes(
    scope: &v8::PinScope<'_, '_>,
    postscript_name: &str,
    family: &str,
) -> Vec<u8> {
    let Some(state) = scope.get_slot::<FontShapingState>() else {
        return Vec::new();
    };
    if let Some(face) = state
        .explicit
        .iter()
        .find(|face| face.family.eq_ignore_ascii_case(family))
    {
        return face.data.as_ref().clone();
    }
    let Some(database) = state.system.as_deref() else {
        return Vec::new();
    };
    let id = database
        .faces()
        .find(|face| {
            face.post_script_name.eq_ignore_ascii_case(postscript_name)
                || face
                    .families
                    .iter()
                    .any(|candidate| candidate.0.eq_ignore_ascii_case(family))
        })
        .map(|face| face.id);
    id.and_then(|id| loaded_system_face(database, id))
        .map(|face| face.data.as_ref().clone())
        .unwrap_or_default()
}

pub(crate) fn metrics_with_features(
    scope: &v8::PinScope<'_, '_>,
    text: &str,
    css_font: &str,
    direction: rustybuzz::Direction,
    kerning: bool,
    variant_caps: &str,
    stretch: &str,
) -> Option<ShapeMetrics> {
    let mut request = FontRequest::parse(css_font);
    if let Some(value) = descriptor_stretch(stretch) {
        request.stretch = value;
    }
    // An empty Canvas string has no advance/ink, but Edge still exposes the
    // selected face's fontBoundingBox* and baseline values.
    let lookup_text = if text.is_empty() { " " } else { text };
    let faces = resolve_faces(scope, &request, lookup_text);
    if faces.is_empty() {
        return None;
    }
    if text.is_empty() {
        let features = shaping_features(&faces[0], kerning, variant_caps);
        return shape_face_with_features(&faces[0], "", request.size, direction, &features);
    }
    // Blink does not kern through a collapsible ASCII-space boundary.  It
    // still shapes every non-space word as one run (so AV/To/ligatures keep
    // their OpenType behavior).
    shape_with_fallback(
        &faces,
        text,
        request.size,
        resolved_text_direction(text, direction),
        true,
        kerning,
        variant_caps,
    )
}

/// DOM layout uses the same face selection and word-boundary shaping as
/// Canvas; the separate entry point keeps DOM callers explicit.
pub(crate) fn dom_metrics(
    scope: &v8::PinScope<'_, '_>,
    text: &str,
    css_font: &str,
    direction: rustybuzz::Direction,
) -> Option<ShapeMetrics> {
    let request = FontRequest::parse(css_font);
    let lookup_text = if text.is_empty() { " " } else { text };
    let faces = resolve_faces(scope, &request, lookup_text);
    if faces.is_empty() {
        return None;
    }
    if text.is_empty() {
        return shape_face(&faces[0], "", request.size, direction);
    }
    shape_with_fallback(
        &faces,
        text,
        request.size,
        resolved_text_direction(text, direction),
        true,
        true,
        "normal",
    )
}

fn shaping_features(
    face: &LoadedFace,
    kerning: bool,
    variant_caps: &str,
) -> Vec<rustybuzz::Feature> {
    let mut output = Vec::new();
    if !kerning {
        output.push(rustybuzz::Feature::new(
            rustybuzz::ttf_parser::Tag::from_bytes(b"kern"),
            0,
            ..,
        ));
    }
    let mut enable = |tag: &[u8; 4]| {
        output.push(rustybuzz::Feature::new(
            rustybuzz::ttf_parser::Tag::from_bytes(tag),
            1,
            ..,
        ));
    };
    match variant_caps {
        "small-caps" => enable(b"smcp"),
        "all-small-caps" => {
            enable(b"smcp");
            enable(b"c2sc");
        }
        "petite-caps" if face_supports_feature(face, b"pcap") => enable(b"pcap"),
        "petite-caps" => enable(b"smcp"),
        "all-petite-caps" => {
            if face_supports_feature(face, b"pcap") {
                enable(b"pcap");
                enable(b"c2pc");
            } else {
                enable(b"smcp");
                enable(b"c2sc");
            }
        }
        "unicase" => enable(b"unic"),
        "titling-caps" => enable(b"titl"),
        _ => {}
    }
    output
}

fn face_supports_feature(face: &LoadedFace, tag: &[u8; 4]) -> bool {
    rustybuzz::Face::from_slice(face.data.as_slice(), face.face_index)
        .and_then(|face| face.tables().gsub)
        .is_some_and(|table| {
            table
                .features
                .find(rustybuzz::ttf_parser::Tag::from_bytes(tag))
                .is_some()
        })
}

fn resolve_faces(
    scope: &v8::PinScope<'_, '_>,
    request: &FontRequest,
    text: &str,
) -> Vec<LoadedFace> {
    let mut output = Vec::new();
    let realm_id = crate::webidl::realm_id(scope);
    let (explicit, dynamic) = scope
        .get_slot::<FontShapingState>()
        .map(|state| {
            (
                state.explicit.clone(),
                state
                    .dynamic
                    .borrow()
                    .iter()
                    .filter(|entry| entry.realm_id == realm_id)
                    .map(|entry| entry.face.clone())
                    .collect::<Vec<_>>(),
            )
        })
        .unwrap_or_default();
    for family in &request.families {
        if let Some(face) = best_explicit_face(&dynamic, family, request) {
            push_unique(&mut output, face);
        }
        if let Some(face) = best_explicit_face(&explicit, family, request) {
            push_unique(&mut output, face);
        }
    }

    let system_enabled = scope
        .get_slot::<FontShapingState>()
        .is_some_and(|state| state.system.is_some());
    if system_enabled {
        for family in &request.families {
            if let Some(face) = system_face(scope, family, request) {
                push_unique(&mut output, face);
            }
        }
        // Blink exhausts CSS families before system fallback.  Build a
        // deterministic fallback list in database order, adding only faces
        // that cover a code point still missing from the selected list.
        let missing = text
            .chars()
            .filter(|character| !faces_cover(&output, *character))
            .collect::<Vec<_>>();
        for character in missing {
            if let Some(face) = system_fallback_face(scope, output.first(), character, request) {
                push_unique(&mut output, face);
            }
        }
    }
    output
}

/// Canvas `direction` controls alignment, but Blink still resolves the
/// shaping direction from the first strong character when a run has an
/// intrinsic bidi direction.  Without this, Arabic measured in the default
/// LTR canvas is emitted as four isolated forms instead of one joined run.
fn resolved_text_direction(text: &str, fallback: rustybuzz::Direction) -> rustybuzz::Direction {
    let bidi = unicode_bidi::BidiInfo::new(text, None);
    bidi.paragraphs
        .first()
        .map(|paragraph| {
            if paragraph.level.is_rtl() {
                rustybuzz::Direction::RightToLeft
            } else {
                rustybuzz::Direction::LeftToRight
            }
        })
        .unwrap_or(fallback)
}

fn push_unique(output: &mut Vec<LoadedFace>, candidate: LoadedFace) {
    if !output.iter().any(|face| {
        Arc::ptr_eq(&face.data, &candidate.data) && face.face_index == candidate.face_index
    }) {
        output.push(candidate);
    }
}

fn best_explicit_face(
    faces: &[LoadedFace],
    family: &str,
    request: &FontRequest,
) -> Option<LoadedFace> {
    faces
        .iter()
        .filter(|face| face.family.eq_ignore_ascii_case(family))
        .min_by_key(|face| trait_distance(face, request))
        .cloned()
}

fn system_face(
    scope: &v8::PinScope<'_, '_>,
    family: &str,
    request: &FontRequest,
) -> Option<LoadedFace> {
    let key = format!(
        "{}|{:?}|{}|{:?}",
        family.to_ascii_lowercase(),
        request.style,
        request.weight.0,
        request.stretch
    );
    if let Some(cached) = scope
        .get_slot::<FontShapingState>()?
        .system_cache
        .borrow()
        .get(&key)
        .cloned()
    {
        return cached;
    }
    let face = {
        let database = scope.get_slot::<FontShapingState>()?.system.as_ref()?;
        let generic = generic_family(family);
        let named;
        let family = if let Some(generic) = generic {
            generic
        } else {
            named = fontdb::Family::Name(family);
            named
        };
        let families = [family];
        let id = database.query(&fontdb::Query {
            families: &families,
            weight: request.weight,
            stretch: request.stretch,
            style: request.style,
        });
        id.and_then(|id| loaded_system_face(database, id))
    };
    scope
        .get_slot::<FontShapingState>()?
        .system_cache
        .borrow_mut()
        .insert(key, face.clone());
    face
}

fn system_fallback_face(
    scope: &v8::PinScope<'_, '_>,
    primary: Option<&LoadedFace>,
    character: char,
    request: &FontRequest,
) -> Option<LoadedFace> {
    let locale = crate::fingerprint::edge(scope).locale.locale.clone();
    // Native fallback selection depends on the base face and locale as well
    // as the missing character.  Omitting either leaks a cached result across
    // unrelated CSS families (or a reconfigured locale in a reused isolate).
    let primary_key = primary
        .map(|face| format!("{}#{}", face.family.to_ascii_lowercase(), face.face_index))
        .unwrap_or_default();
    let key = format!(
        "{}|{}|{}|{:?}|{}|{:?}",
        character,
        locale.to_ascii_lowercase(),
        primary_key,
        request.style,
        request.weight.0,
        request.stretch
    );
    if let Some(cached) = scope
        .get_slot::<FontShapingState>()?
        .fallback_cache
        .borrow()
        .get(&key)
        .cloned()
    {
        return cached;
    }
    // Chromium's Windows fallback is script-driven before it reaches the
    // DirectWrite fallback mapper. In particular, Han and kana use the
    // non-UI CJK families (Microsoft YaHei / Yu Gothic / MS PGothic), whose
    // TTC member and full-em advances differ from the UI face selected by a
    // direct IDWriteFontFallback query.
    for family in script_fallback_families(character, &locale) {
        if let Some(face) = system_face(scope, family, request)
            && grapheme_supported(&face, &character.to_string())
        {
            scope
                .get_slot::<FontShapingState>()?
                .fallback_cache
                .borrow_mut()
                .insert(key, Some(face.clone()));
            return Some(face);
        }
    }
    // On Windows/macOS ask the same native text stack that Chromium delegates
    // fallback to.  font-kit's Linux FreeType implementation intentionally
    // returns no candidates, so Linux continues into the fontdb coverage path.
    if let Some(native) = primary.and_then(|face| face.native_font.as_deref()) {
        use font_kit::loader::Loader;
        let fallback = native.get_fallbacks(&character.to_string(), &locale);
        if let Some(candidate) = fallback.fonts.first() {
            // Keep the exact native fallback face. A family-name round trip
            // is insufficient for TTC collections: DirectWrite can select
            // Microsoft YaHei UI's collection member while fontdb's generic
            // query returns the first member with different glyph metrics.
            if let Some(mut loaded) = loaded_native_fallback_face(&candidate.font, character) {
                loaded.size_scale = f64::from(candidate.scale);
                scope
                    .get_slot::<FontShapingState>()?
                    .fallback_cache
                    .borrow_mut()
                    .insert(key, Some(loaded.clone()));
                return Some(loaded);
            }
        }
    }
    let database = scope.get_slot::<FontShapingState>()?.system.as_ref()?;
    let mut candidates = database
        .faces()
        .filter(|info| {
            database
                .with_face_data(info.id, |data, index| {
                    rustybuzz::Face::from_slice(data, index)
                        .and_then(|face| face.glyph_index(character))
                        .is_some()
                })
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();
    candidates.sort_by_key(|face| {
        (
            face.style != request.style,
            face.weight.0.abs_diff(request.weight.0),
            format!("{:?}", face.stretch != request.stretch),
            face.families
                .first()
                .map(|family| family.0.to_ascii_lowercase())
                .unwrap_or_default(),
            face.index,
        )
    });
    let face = candidates
        .first()
        .and_then(|face| loaded_system_face(database, face.id));
    scope
        .get_slot::<FontShapingState>()?
        .fallback_cache
        .borrow_mut()
        .insert(key, face.clone());
    face
}

fn script_fallback_families(character: char, locale: &str) -> &'static [&'static str] {
    // These tables are the Windows-specific Blink fallback map. CoreText and
    // fontconfig have their own platform fallback order and must not inherit
    // Windows family preferences merely because similarly named fonts happen
    // to be installed.
    #[cfg(not(target_os = "windows"))]
    {
        let _ = (character, locale);
        return &[];
    }
    #[cfg(target_os = "windows")]
    {
        let code = character as u32;
        if is_hiragana_or_katakana(code) {
            // Edge 150's stable Windows configuration has
            // FontSystemFallbackNotoCjk disabled, so Blink selects from its
            // no-Noto list even if optional Noto CJK fonts are installed.
            return &["Meiryo", "Yu Gothic", "MS PGothic", "Microsoft YaHei"];
        }
        if is_han(code) {
            let locale = locale.to_ascii_lowercase();
            if locale.starts_with("zh-tw")
                || locale.starts_with("zh-hk")
                || locale.starts_with("zh-mo")
            {
                return &["Microsoft JhengHei", "PMingLiU"];
            }
            if locale.starts_with("ja") {
                return &["Meiryo", "Yu Gothic", "MS PGothic", "Microsoft YaHei"];
            }
            if locale.starts_with("ko") {
                return &["Malgun Gothic", "Gulim"];
            }
            return &["Microsoft YaHei", "SimSun"];
        }
        &[]
    }
}

fn is_hiragana_or_katakana(code: u32) -> bool {
    matches!(code, 0x3040..=0x30FF | 0x31F0..=0x31FF | 0xFF66..=0xFF9D)
}

fn is_han(code: u32) -> bool {
    matches!(
        code,
        0x3400..=0x4DBF
            | 0x4E00..=0x9FFF
            | 0xF900..=0xFAFF
            | 0x20000..=0x2FA1F
            | 0x30000..=0x323AF
    )
}

fn loaded_native_fallback_face(
    native: &font_kit::font::Font,
    requested_character: char,
) -> Option<LoadedFace> {
    let data = native.copy_font_data()?;
    let native_properties = native.properties();
    let style = if native_properties.style == font_kit::properties::Style::Italic {
        fontdb::Style::Italic
    } else if native_properties.style == font_kit::properties::Style::Oblique {
        fontdb::Style::Oblique
    } else {
        fontdb::Style::Normal
    };
    let weight = fontdb::Weight(native_properties.weight.0.round().clamp(1.0, 1000.0) as u16);
    let stretch = fontdb::Stretch::Normal;
    let native_character_index = |character: char| native.glyph_for_char(character);
    let mut face_index = None;
    // Match the native face inside a TTC/OTC by its glyph map. This avoids
    // assuming that the family member index equals the collection index.
    for index in 0..256_u32 {
        let Some(face) = rustybuzz::Face::from_slice(data.as_slice(), index) else {
            break;
        };
        if face
            .glyph_index(requested_character)
            .map(|glyph| u32::from(glyph.0))
            == native_character_index(requested_character)
        {
            face_index = Some(index);
            break;
        }
    }
    Some(LoadedFace {
        family: native.family_name(),
        native_font: Some(Rc::new(native.clone())),
        data,
        face_index: face_index?,
        style,
        weight,
        stretch,
        platform_face: true,
        size_scale: 1.0,
    })
}

fn loaded_system_face(database: &fontdb::Database, id: fontdb::ID) -> Option<LoadedFace> {
    let info = database.face(id)?;
    let family = info.families.first()?.0.clone();
    let style = info.style;
    let weight = info.weight;
    let stretch = info.stretch;
    let face_index = info.index;
    let data = database.with_face_data(id, |bytes, _| Arc::new(bytes.to_vec()))?;
    Some(LoadedFace {
        family,
        native_font: native_font(data.clone(), face_index),
        size_scale: 1.0,
        data,
        face_index,
        style,
        weight,
        stretch,
        platform_face: true,
    })
}

fn faces_cover(faces: &[LoadedFace], character: char) -> bool {
    faces.iter().any(|loaded| {
        rustybuzz::Face::from_slice(loaded.data.as_slice(), loaded.face_index)
            .and_then(|face| face.glyph_index(character))
            .is_some()
    })
}

fn shape_with_fallback(
    faces: &[LoadedFace],
    text: &str,
    font_size: f64,
    direction: rustybuzz::Direction,
    split_ascii_spaces: bool,
    kerning: bool,
    variant_caps: &str,
) -> Option<ShapeMetrics> {
    let mut output = ShapeMetrics::default();
    let mut has_output = false;
    let mut run = String::new();
    let mut run_face = None;
    let flush = |run: &mut String,
                 run_face: &mut Option<usize>,
                 output: &mut ShapeMetrics,
                 has_output: &mut bool|
     -> Option<()> {
        if run.is_empty() {
            return Some(());
        }
        let index = run_face.take()?;
        let features = shaping_features(&faces[index], kerning, variant_caps);
        let shaped = shape_face_with_features(&faces[index], run, font_size, direction, &features)?;
        if *has_output {
            merge_metrics(output, shaped);
        } else {
            *output = shaped;
            *has_output = true;
        }
        run.clear();
        Some(())
    };
    for grapheme in unicode_segmentation::UnicodeSegmentation::graphemes(text, true) {
        if split_ascii_spaces && grapheme == " " {
            flush(&mut run, &mut run_face, &mut output, &mut has_output)?;
            let index = faces
                .iter()
                .position(|face| grapheme_supported(face, grapheme))
                .unwrap_or(0);
            let features = shaping_features(&faces[index], kerning, variant_caps);
            let shaped =
                shape_face_with_features(&faces[index], grapheme, font_size, direction, &features)?;
            if has_output {
                merge_metrics(&mut output, shaped);
            } else {
                output = shaped;
                has_output = true;
            }
            continue;
        }
        let index = faces
            .iter()
            .position(|face| grapheme_supported(face, grapheme))
            .unwrap_or(0);
        if run_face.is_some_and(|current| current != index) {
            flush(&mut run, &mut run_face, &mut output, &mut has_output)?;
        }
        run_face = Some(index);
        run.push_str(grapheme);
    }
    flush(&mut run, &mut run_face, &mut output, &mut has_output)?;
    // TextMetrics.fontBoundingBox* describes the CSS font used for the run,
    // not a larger fallback face selected for an individual code point.
    if let Some(primary) = faces.first()
        && let Some(face) = rustybuzz::Face::from_slice(primary.data.as_slice(), primary.face_index)
    {
        (output.font_ascent, output.font_descent) =
            edge_font_box(face.as_ref(), font_size * primary.size_scale);
    }
    Some(output)
}

/// Shapes a complete inline run and returns one logical metric record per
/// supplied grapheme.  Glyph clusters that cover multiple graphemes (for
/// example an OpenType ligature) share their advance between those logical
/// graphemes so DOM Range boundaries remain addressable.
pub(crate) fn grapheme_metrics(
    scope: &v8::PinScope<'_, '_>,
    graphemes: &[&str],
    css_font: &str,
    direction: rustybuzz::Direction,
) -> Option<Vec<ShapeMetrics>> {
    if graphemes.is_empty() {
        return Some(Vec::new());
    }
    let text = graphemes.concat();
    let request = FontRequest::parse(css_font);
    let faces = resolve_faces(scope, &request, &text);
    if faces.is_empty() {
        return None;
    }
    let mut output = vec![ShapeMetrics::default(); graphemes.len()];
    let mut run_start = 0_usize;
    let mut run_face = None;
    for (index, grapheme) in graphemes.iter().enumerate() {
        let face = faces
            .iter()
            .position(|face| grapheme_supported(face, grapheme))
            .unwrap_or(0);
        if run_face.is_some_and(|current| current != face) {
            shape_grapheme_face_run(
                &faces[run_face?],
                &graphemes[run_start..index],
                request.size,
                direction,
                &mut output[run_start..index],
            )?;
            run_start = index;
        }
        run_face = Some(face);
    }
    shape_grapheme_face_run(
        &faces[run_face?],
        &graphemes[run_start..],
        request.size,
        direction,
        &mut output[run_start..],
    )?;
    Some(output)
}

fn shape_grapheme_face_run(
    loaded: &LoadedFace,
    graphemes: &[&str],
    font_size: f64,
    direction: rustybuzz::Direction,
    output: &mut [ShapeMetrics],
) -> Option<()> {
    let text = graphemes.concat();
    let face = rustybuzz::Face::from_slice(loaded.data.as_slice(), loaded.face_index)?;
    let units = f64::from(face.units_per_em());
    if units <= 0.0 {
        return None;
    }
    let font_size = font_size * loaded.size_scale;
    let scale = font_size / units;
    let mut byte_starts = Vec::with_capacity(graphemes.len() + 1);
    let mut byte = 0_usize;
    for grapheme in graphemes {
        byte_starts.push(byte);
        byte += grapheme.len();
    }
    byte_starts.push(byte);
    let mut buffer = rustybuzz::UnicodeBuffer::new();
    buffer.push_str(&text);
    buffer.set_cluster_level(rustybuzz::BufferClusterLevel::MonotoneCharacters);
    buffer.set_direction(direction);
    buffer.guess_segment_properties();
    let shaped = rustybuzz::shape(&face, &[], buffer);
    let mut cluster_advances = HashMap::<usize, f64>::new();
    let mut cluster_glyph_count = HashMap::<usize, usize>::new();
    let mut cluster_top = HashMap::<usize, f64>::new();
    let mut cluster_bottom = HashMap::<usize, f64>::new();
    for (info, position) in shaped.glyph_infos().iter().zip(shaped.glyph_positions()) {
        if info.glyph_id == 0 {
            return None;
        }
        let cluster = info.cluster as usize;
        *cluster_advances.entry(cluster).or_default() +=
            floor_layout_unit(f64::from(position.x_advance).abs() * scale);
        *cluster_glyph_count.entry(cluster).or_default() += 1;
        if let Some(bounds) =
            face.glyph_bounding_box(rustybuzz::ttf_parser::GlyphId(info.glyph_id as u16))
        {
            cluster_top
                .entry(cluster)
                .and_modify(|top| {
                    *top = top.max(f64::from(position.y_offset + i32::from(bounds.y_max)))
                })
                .or_insert(f64::from(position.y_offset + i32::from(bounds.y_max)));
            cluster_bottom
                .entry(cluster)
                .and_modify(|bottom| {
                    *bottom = bottom.min(f64::from(position.y_offset + i32::from(bounds.y_min)))
                })
                .or_insert(f64::from(position.y_offset + i32::from(bounds.y_min)));
        }
    }
    let mut clusters = cluster_advances.keys().copied().collect::<Vec<_>>();
    clusters.sort_unstable();
    let face_ref = face.as_ref();
    let (font_ascent, font_descent) = edge_font_box(face_ref, font_size);
    for (cluster_index, cluster_start) in clusters.iter().copied().enumerate() {
        let cluster_end = clusters
            .get(cluster_index + 1)
            .copied()
            .unwrap_or(text.len());
        let first = byte_starts
            .partition_point(|start| *start <= cluster_start)
            .saturating_sub(1)
            .min(graphemes.len() - 1);
        let last_exclusive = byte_starts
            .partition_point(|start| *start < cluster_end)
            .max(first + 1)
            .min(graphemes.len());
        let logical_count = last_exclusive - first;
        let cluster_advance = cluster_advances[&cluster_start];
        let ascent = cluster_top
            .get(&cluster_start)
            .copied()
            .unwrap_or(0.0)
            .max(0.0)
            * scale;
        let descent =
            (-cluster_bottom.get(&cluster_start).copied().unwrap_or(0.0)).max(0.0) * scale;
        let allocation = if logical_count > 1
            && cluster_glyph_count
                .get(&cluster_start)
                .copied()
                .unwrap_or(0)
                == 1
        {
            // Blink's editing geometry distributes a ligature using the
            // unligated component advances, scaled to the shaped cluster.
            // This keeps every DOM caret addressable while preserving the
            // HarfBuzz/DirectWrite total run width.
            let singles = graphemes[first..last_exclusive]
                .iter()
                .map(|grapheme| {
                    shape_face(loaded, grapheme, font_size, direction)
                        .map(|metrics| metrics.advance)
                        .unwrap_or(0.0)
                })
                .collect::<Vec<_>>();
            let total = singles.iter().sum::<f64>();
            if total > 0.0 {
                singles
                    .into_iter()
                    .map(|advance| advance * cluster_advance / total)
                    .collect::<Vec<_>>()
            } else {
                vec![cluster_advance / logical_count as f64; logical_count]
            }
        } else {
            vec![cluster_advance / logical_count as f64; logical_count]
        };
        for (metric, advance) in output[first..last_exclusive].iter_mut().zip(allocation) {
            metric.advance += advance;
            metric.actual_right = metric.advance;
            metric.font_ascent = font_ascent;
            metric.font_descent = font_descent;
            metric.actual_ascent = metric.actual_ascent.max(ascent);
            metric.actual_descent = metric.actual_descent.max(descent);
        }
    }
    Some(())
}

fn grapheme_supported(face: &LoadedFace, grapheme: &str) -> bool {
    let Some(face) = rustybuzz::Face::from_slice(face.data.as_slice(), face.face_index) else {
        return false;
    };
    grapheme
        .chars()
        .filter(|character| !is_default_ignorable(*character))
        .all(|character| face.glyph_index(character).is_some())
}

fn is_default_ignorable(character: char) -> bool {
    matches!(character, '\u{200C}' | '\u{200D}' | '\u{FE0E}' | '\u{FE0F}')
        || matches!(character as u32, 0xE0020..=0xE007F)
}

fn shape_face(
    loaded: &LoadedFace,
    text: &str,
    font_size: f64,
    direction: rustybuzz::Direction,
) -> Option<ShapeMetrics> {
    shape_face_with_features(loaded, text, font_size, direction, &[])
}

fn shape_face_with_features(
    loaded: &LoadedFace,
    text: &str,
    font_size: f64,
    direction: rustybuzz::Direction,
    features: &[rustybuzz::Feature],
) -> Option<ShapeMetrics> {
    let face = rustybuzz::Face::from_slice(loaded.data.as_slice(), loaded.face_index)?;
    let font_size = font_size * loaded.size_scale;
    let units = f64::from(face.units_per_em());
    if units <= 0.0 {
        return None;
    }
    let scale = font_size / units;
    let mut buffer = rustybuzz::UnicodeBuffer::new();
    buffer.push_str(text);
    buffer.set_cluster_level(rustybuzz::BufferClusterLevel::MonotoneCharacters);
    buffer.set_direction(direction);
    buffer.guess_segment_properties();
    let glyphs = rustybuzz::shape(&face, features, buffer);
    let positions = glyphs.glyph_positions();
    let infos = glyphs.glyph_infos();
    let platform_font = loaded.native_font.as_deref();
    let mut pen_x = 0.0;
    let mut pen_y = 0.0;
    let mut minimum_x = f64::INFINITY;
    let mut maximum_x = f64::NEG_INFINITY;
    let mut maximum_y = f64::NEG_INFINITY;
    let mut minimum_y = f64::INFINITY;
    for (info, position) in infos.iter().zip(positions) {
        if info.glyph_id == 0 {
            return None;
        }
        let glyph = rustybuzz::ttf_parser::GlyphId(info.glyph_id as u16);
        let raster_bounds = platform_font.and_then(|font| {
            use font_kit::canvas::RasterizationOptions;
            use font_kit::hinting::HintingOptions;
            use pathfinder_geometry::transform2d::Transform2F;
            font.raster_bounds(
                info.glyph_id,
                font_size as f32,
                Transform2F::default(),
                HintingOptions::VerticalSubpixel(font_size as f32),
                RasterizationOptions::SubpixelAa,
            )
            .ok()
            .map(|bounds| {
                (
                    f64::from(bounds.min_x()),
                    f64::from(bounds.max_x()),
                    f64::from(-bounds.max_y()),
                    f64::from(-bounds.min_y()),
                )
            })
        });
        let outline_bounds = face.glyph_bounding_box(glyph).map(|bounds| {
            (
                f64::from(bounds.x_min) * scale,
                f64::from(bounds.x_max) * scale,
                f64::from(bounds.y_min) * scale,
                f64::from(bounds.y_max) * scale,
            )
        });
        if let Some((glyph_left, glyph_right, glyph_bottom, glyph_top)) =
            raster_bounds.or(outline_bounds).map(|raster| {
                // DirectWrite/CoreText/FreeType return a device-space raster
                // rectangle. Canvas exposes its signed leading distance (the
                // negation happens below) and the trailing/vertical edges.
                (raster.0, raster.1, raster.2, raster.3)
            })
        {
            let x_offset = quantized_layout_delta(f64::from(position.x_offset) * scale);
            let y_offset = quantized_layout_delta(f64::from(position.y_offset) * scale);
            let left = pen_x + x_offset + glyph_left;
            let right = pen_x + x_offset + glyph_right;
            let bottom = pen_y + y_offset + glyph_bottom;
            let top = pen_y + y_offset + glyph_top;
            minimum_x = minimum_x.min(left);
            maximum_x = maximum_x.max(right);
            minimum_y = minimum_y.min(bottom);
            maximum_y = maximum_y.max(top);
        }
        pen_x += quantized_layout_delta(f64::from(position.x_advance) * scale);
        pen_y += quantized_layout_delta(f64::from(position.y_advance) * scale);
    }
    let face = face.as_ref();
    // Blink stores shaped advances in a 16.16 layout unit.  Flooring at the
    // run boundary is observable for fractional CSS sizes (13.3333px is
    // normalized to 13.33px, then e.g. Arial H becomes 9.626495361328125).
    let advance = pen_x.abs();
    let (font_ascent, font_descent) = edge_font_box(face, font_size);
    Some(ShapeMetrics {
        advance,
        actual_left: if minimum_x.is_finite() {
            -minimum_x
        } else {
            0.0
        },
        actual_right: if maximum_x.is_finite() {
            maximum_x
        } else {
            advance
        },
        font_ascent,
        font_descent,
        actual_ascent: if maximum_y.is_finite() {
            maximum_y.max(0.0)
        } else {
            0.0
        },
        actual_descent: if minimum_y.is_finite() {
            (-minimum_y).max(0.0)
        } else {
            0.0
        },
        platform_face: loaded.platform_face,
    })
}

fn edge_font_box(face: &rustybuzz::ttf_parser::Face<'_>, font_size: f64) -> (f64, f64) {
    let units = f64::from(face.units_per_em());
    if units <= 0.0 {
        return (0.0, 0.0);
    }
    // Chromium exposes the raster font box at integral CSS-pixel boundaries.
    // DirectWrite/CoreText/fontconfig may select different faces, but once a
    // face is selected both sides are rounded to integral CSS-pixel
    // boundaries. This matches Edge's Arial, Times New Roman and Segoe UI
    // evidence at 10/13.3333/16/48px.
    (
        (f64::from(face.ascender()).max(0.0) * font_size / units).round(),
        (f64::from(-face.descender()).max(0.0) * font_size / units).round(),
    )
}

fn native_font(data: Arc<Vec<u8>>, face_index: u32) -> Option<Rc<font_kit::font::Font>> {
    font_kit::font::Font::from_bytes(data, face_index)
        .ok()
        .map(Rc::new)
}

fn floor_layout_unit(value: f64) -> f64 {
    (value * 65_536.0).floor() / 65_536.0
}

fn quantized_layout_delta(value: f64) -> f64 {
    if value.is_sign_negative() {
        -floor_layout_unit(-value)
    } else {
        floor_layout_unit(value)
    }
}

fn merge_metrics(output: &mut ShapeMetrics, run: ShapeMetrics) {
    let offset = output.advance;
    let run_left = offset - run.actual_left;
    let run_right = offset + run.actual_right;
    let old_left = -output.actual_left;
    let old_right = output.actual_right;
    output.actual_left = -old_left.min(run_left);
    output.actual_right = old_right.max(run_right);
    output.advance += run.advance;
    output.font_ascent = output.font_ascent.max(run.font_ascent);
    output.font_descent = output.font_descent.max(run.font_descent);
    output.actual_ascent = output.actual_ascent.max(run.actual_ascent);
    output.actual_descent = output.actual_descent.max(run.actual_descent);
    output.platform_face |= run.platform_face;
}

fn trait_distance(face: &LoadedFace, request: &FontRequest) -> (bool, u16, bool) {
    (
        face.style != request.style,
        face.weight.0.abs_diff(request.weight.0),
        face.stretch != request.stretch,
    )
}

fn face_traits(
    face: &rustybuzz::ttf_parser::Face<'_>,
) -> (fontdb::Style, fontdb::Weight, fontdb::Stretch) {
    let style = if face.is_italic() {
        fontdb::Style::Italic
    } else {
        fontdb::Style::Normal
    };
    let weight = fontdb::Weight(face.weight().to_number());
    (style, weight, fontdb::Stretch::Normal)
}

fn descriptor_style(value: &str) -> Option<fontdb::Style> {
    match value.trim().to_ascii_lowercase().as_str() {
        "normal" => Some(fontdb::Style::Normal),
        "italic" => Some(fontdb::Style::Italic),
        value if value.starts_with("oblique") => Some(fontdb::Style::Oblique),
        _ => None,
    }
}

fn descriptor_weight(value: &str) -> Option<fontdb::Weight> {
    match value.trim().to_ascii_lowercase().as_str() {
        "normal" => Some(fontdb::Weight::NORMAL),
        "bold" => Some(fontdb::Weight::BOLD),
        value => value
            .parse::<u16>()
            .ok()
            .filter(|weight| (1..=1000).contains(weight))
            .map(fontdb::Weight),
    }
}

fn descriptor_stretch(value: &str) -> Option<fontdb::Stretch> {
    match value.trim().to_ascii_lowercase().as_str() {
        "ultra-condensed" => Some(fontdb::Stretch::UltraCondensed),
        "extra-condensed" => Some(fontdb::Stretch::ExtraCondensed),
        "condensed" => Some(fontdb::Stretch::Condensed),
        "semi-condensed" => Some(fontdb::Stretch::SemiCondensed),
        "normal" => Some(fontdb::Stretch::Normal),
        "semi-expanded" => Some(fontdb::Stretch::SemiExpanded),
        "expanded" => Some(fontdb::Stretch::Expanded),
        "extra-expanded" => Some(fontdb::Stretch::ExtraExpanded),
        "ultra-expanded" => Some(fontdb::Stretch::UltraExpanded),
        _ => None,
    }
}

#[derive(Clone)]
struct FontRequest {
    size: f64,
    families: Vec<String>,
    style: fontdb::Style,
    weight: fontdb::Weight,
    stretch: fontdb::Stretch,
}

impl FontRequest {
    fn parse(font: &str) -> Self {
        let size = crate::web::offscreen_canvas_rendering_context_2d::canvas_font_size(font);
        let marker = font.split_ascii_whitespace().find(|part| {
            part.trim_end_matches(|character: char| character.is_ascii_alphabetic())
                .parse::<f64>()
                .is_ok()
                && part.to_ascii_lowercase().contains("px")
        });
        let family_source = marker
            .and_then(|marker| font.find(marker).map(|index| &font[index + marker.len()..]))
            .unwrap_or("sans-serif");
        let families = split_families(family_source);
        let lower = font.to_ascii_lowercase();
        let style = if lower.contains("italic") {
            fontdb::Style::Italic
        } else if lower.contains("oblique") {
            fontdb::Style::Oblique
        } else {
            fontdb::Style::Normal
        };
        let weight = lower
            .split_ascii_whitespace()
            .find_map(|token| {
                token
                    .parse::<u16>()
                    .ok()
                    .filter(|value| (1..=1000).contains(value))
            })
            .map(fontdb::Weight)
            .unwrap_or_else(|| {
                if lower.contains("bold") {
                    fontdb::Weight::BOLD
                } else {
                    fontdb::Weight::NORMAL
                }
            });
        Self {
            size,
            families: if families.is_empty() {
                vec!["sans-serif".to_owned()]
            } else {
                families
            },
            style,
            weight,
            stretch: lower
                .split_ascii_whitespace()
                .find_map(descriptor_stretch)
                .unwrap_or(fontdb::Stretch::Normal),
        }
    }
}

fn split_families(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(|family| family.trim().trim_matches(['\'', '"']).to_owned())
        .filter(|family| !family.is_empty())
        .collect()
}

fn generic_family(value: &str) -> Option<fontdb::Family<'static>> {
    match value.trim().to_ascii_lowercase().as_str() {
        "serif" => Some(fontdb::Family::Serif),
        "sans-serif" | "system-ui" | "ui-sans-serif" => Some(fontdb::Family::SansSerif),
        "monospace" | "ui-monospace" => Some(fontdb::Family::Monospace),
        "cursive" => Some(fontdb::Family::Cursive),
        "fantasy" => Some(fontdb::Family::Fantasy),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{FontRequest, split_families};

    #[test]
    fn parses_css_family_lists_without_losing_quoted_names() {
        assert_eq!(
            split_families("\"Times New Roman\", Arial, serif"),
            ["Times New Roman", "Arial", "serif"]
        );
        let request = FontRequest::parse("italic 700 16px \"Times New Roman\", serif");
        assert_eq!(request.size, 16.0);
        assert_eq!(request.families, ["Times New Roman", "serif"]);
        assert_eq!(request.weight, fontdb::Weight::BOLD);
        assert_eq!(request.style, fontdb::Style::Italic);
    }
}
