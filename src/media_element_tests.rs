use crate::{EdgeRuntime, Evaluation};

fn text(runtime: &mut EdgeRuntime, source: &str) -> String {
    match runtime.evaluate(source).expect("JavaScript evaluation") {
        Evaluation::String(value) | Evaluation::Number(value) | Evaluation::Other(value) => value,
        Evaluation::Boolean(value) => value.to_string(),
        Evaluation::Undefined => "undefined".to_owned(),
        Evaluation::Null => "null".to_owned(),
    }
}

fn pcm_wav_data_url(sample_rate: u32, sample_count: u32) -> String {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(36 + sample_count).to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16u32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&sample_rate.to_le_bytes());
    bytes.extend_from_slice(&sample_rate.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&8u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&sample_count.to_le_bytes());
    bytes.resize(bytes.len() + sample_count as usize, 128);
    format!("data:audio/wav;base64,{}", base64(&bytes))
}

fn base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut output = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let first = chunk[0];
        let second = chunk.get(1).copied().unwrap_or(0);
        let third = chunk.get(2).copied().unwrap_or(0);
        output.push(char::from(ALPHABET[usize::from(first >> 2)]));
        output.push(char::from(
            ALPHABET[usize::from((first & 0x03) << 4 | second >> 4)],
        ));
        output.push(if chunk.len() > 1 {
            char::from(ALPHABET[usize::from((second & 0x0f) << 2 | third >> 6)])
        } else {
            '='
        });
        output.push(if chunk.len() > 2 {
            char::from(ALPHABET[usize::from(third & 0x3f)])
        } else {
            '='
        });
    }
    output
}

#[test]
fn audio_data_url_loads_metadata_asynchronously_without_decoding() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let data_url = pcm_wav_data_url(8_000, 12_896);
    let source = format!(
        r#"
        new Promise(resolve => {{
          const audio = new Audio();
          const order = ["sync-before"];
          let targetMatches = true;
          for (const type of ["loadstart", "durationchange", "loadedmetadata"]) {{
            audio.addEventListener(type, event => {{
              order.push(type);
              targetMatches &&= event.target === audio && event.currentTarget === audio;
            }});
          }}
          audio.onerror = () => resolve("unexpected-error:" + audio.error?.message);
          audio.onloadedmetadata = () => resolve([
            audio.duration.toFixed(3),
            audio.readyState,
            audio.networkState,
            audio.currentSrc.startsWith("data:audio/wav;base64,"),
            audio.error === null,
            audio.contentEditable,
            audio.loading,
            targetMatches,
            order.join(",")
          ].join("|"));
          audio.src = {data_url:?};
          order.push("sync-after");
        }})
        "#,
    );
    assert_eq!(
        text(&mut runtime, &source),
        "1.612|1|1|true|true|inherit|eager|true|sync-before,sync-after,loadstart,durationchange,loadedmetadata"
    );
}

#[test]
fn complete_local_audio_advances_ready_state_and_buffered_after_metadata_dispatch() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let data_url = pcm_wav_data_url(8_000, 4_000);
    let source = format!(
        r#"
        (() => {{
          globalThis.completeAudio = new Audio();
          globalThis.completeAudioEvents = [];
          for (const type of [
            "durationchange", "loadedmetadata", "loadeddata", "canplay", "canplaythrough"
          ]) globalThis.completeAudio.addEventListener(
            type,
            () => globalThis.completeAudioEvents.push(type)
          );
          globalThis.completeAudio.src = {data_url:?};
        }})()
        "#,
    );
    text(&mut runtime, &source);
    assert_eq!(
        text(
            &mut runtime,
            r#"(() => {
              const audio = globalThis.completeAudio;
              const buffered = audio.buffered;
              return [
                audio.readyState,
                audio.networkState,
                audio.duration,
                buffered.length,
                buffered.length ? buffered.start(0) : "empty",
                buffered.length ? buffered.end(0) : "empty",
                globalThis.completeAudioEvents.join(",")
              ].join("|");
            })()"#,
        ),
        "4|1|0.5|1|0|0.5|durationchange,loadedmetadata,loadeddata,canplay,canplaythrough"
    );
}

#[test]
fn playing_media_current_time_advances_and_pause_freezes_the_clock() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let data_url = pcm_wav_data_url(8_000, 80_000);
    let source = format!(
        r#"
        new Promise(resolve => {{
          const audio = new Audio();
          audio.onloadedmetadata = async () => {{
            await audio.play();
            const started = audio.currentTime;
            setTimeout(() => {{
              const advanced = audio.currentTime;
              audio.pause();
              const paused = audio.currentTime;
              setTimeout(() => resolve([
                started,
                advanced,
                paused,
                audio.currentTime
              ].join("|")), 20);
            }}, 25);
          }};
          audio.onerror = () => resolve("error");
          audio.src = {data_url:?};
        }})
        "#,
    );
    let values = text(&mut runtime, &source)
        .split('|')
        .map(str::to_owned)
        .collect::<Vec<_>>();
    assert_eq!(values.len(), 4, "unexpected media result: {values:?}");
    let started = values[0].parse::<f64>().expect("started currentTime");
    let advanced = values[1].parse::<f64>().expect("advanced currentTime");
    let paused = values[2].parse::<f64>().expect("paused currentTime");
    let frozen = values[3].parse::<f64>().expect("frozen currentTime");
    assert!(
        advanced >= started + 0.02,
        "media clock did not advance: {started} -> {advanced}; {values:?}"
    );
    assert!(
        (paused - advanced).abs() < 0.01,
        "pause changed currentTime"
    );
    assert!(
        (frozen - paused).abs() < 0.001,
        "paused media clock advanced"
    );
}

#[test]
fn invalid_audio_data_url_exposes_media_error_and_error_event() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let answer = text(
        &mut runtime,
        r#"
        new Promise(resolve => {
          const audio = new Audio();
          let metadataEvents = 0;
          audio.onloadedmetadata = () => metadataEvents++;
          audio.onerror = event => resolve([
            event.target === audio,
            audio.error instanceof MediaError,
            audio.error.code,
            audio.error.message,
            audio.networkState,
            audio.readyState,
            Number.isNaN(audio.duration),
            metadataEvents
          ].join("|"));
          audio.src = "data:audio/wav;base64,bm90LWFuLWF1ZGlv";
        })
        "#,
    );
    assert_eq!(
        answer,
        "true|true|4|MEDIA_ELEMENT_ERROR: Format error|3|0|true|0"
    );
}

#[test]
fn audio_constructor_source_and_latest_generation_win() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let data_url = pcm_wav_data_url(1_000, 1_612);
    let source = format!(
        r#"
        new Promise(resolve => {{
          const audio = new Audio("data:audio/wav;base64,bm90LWFuLWF1ZGlv");
          let errors = 0;
          audio.onerror = () => errors++;
          audio.onloadedmetadata = () => resolve([
            audio.duration.toFixed(3),
            errors,
            audio.getAttribute("src") === {data_url:?}
          ].join("|"));
          audio.src = {data_url:?};
        }})
        "#,
    );
    assert_eq!(text(&mut runtime, &source), "1.612|0|true");
}

#[test]
fn media_metadata_api_remains_available() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let answer = text(
        &mut runtime,
        r#"
        (() => {
          const metadata = new MediaMetadata({
            title: "Track",
            artist: "Artist",
            album: "Album",
            artwork: [{src: "/cover.png", sizes: "64x64", type: "image/png"}],
            chapterInfo: [{title: "Opening", startTime: "12.5", artwork: []}]
          });
          navigator.mediaSession.metadata = metadata;
          const chapter = metadata.chapterInfo[0];
          return [
            metadata instanceof MediaMetadata,
            navigator.mediaSession.metadata === metadata,
            metadata.title,
            metadata.artist,
            metadata.album,
            metadata.artwork[0].src,
            chapter instanceof ChapterInformation,
            chapter.title,
            chapter.startTime
          ].join("|");
        })()
        "#,
    );
    assert_eq!(
        answer,
        "true|true|Track|Artist|Album|/cover.png|true|Opening|12.5"
    );
}
