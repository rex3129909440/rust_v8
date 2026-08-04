//! Container-level audio metadata parsing without sample decoding.

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MediaMetadata {
    pub(crate) duration: f64,
}

pub(crate) fn parse(bytes: &[u8], _content_type: &str) -> Option<MediaMetadata> {
    let duration = wav_duration(bytes)
        .or_else(|| flac_duration(bytes))
        .or_else(|| ogg_duration(bytes))
        .or_else(|| mp4_duration(bytes))
        .or_else(|| webm_duration(bytes))
        .or_else(|| mp3_duration(bytes))
        .or_else(|| adts_duration(bytes))?;
    (duration.is_finite() && duration >= 0.0).then_some(MediaMetadata { duration })
}

fn wav_duration(bytes: &[u8]) -> Option<f64> {
    if bytes.len() < 12 || !matches!(&bytes[..4], b"RIFF" | b"RF64") || &bytes[8..12] != b"WAVE" {
        return None;
    }
    let rf64 = &bytes[..4] == b"RF64";
    let mut sample_rate = None;
    let mut byte_rate = None;
    let mut fact_samples = None;
    let mut rf64_data_size = None;
    let mut data_size = None;
    let mut offset = 12usize;
    while offset.checked_add(8)? <= bytes.len() {
        let chunk_id = bytes.get(offset..offset + 4)?;
        let declared_size = u32::from_le_bytes(bytes[offset + 4..offset + 8].try_into().ok()?);
        let body = offset + 8;
        let size = if chunk_id == b"data" && rf64 && declared_size == u32::MAX {
            usize::try_from(rf64_data_size?).ok()?
        } else {
            usize::try_from(declared_size).ok()?
        };
        let end = body.checked_add(size)?;
        if end > bytes.len() {
            return None;
        }
        match chunk_id {
            b"ds64" if size >= 16 => {
                rf64_data_size = Some(u64::from_le_bytes(
                    bytes[body + 8..body + 16].try_into().ok()?,
                ));
            }
            b"fmt " if size >= 16 => {
                sample_rate = Some(u32::from_le_bytes(
                    bytes[body + 4..body + 8].try_into().ok()?,
                ));
                byte_rate = Some(u32::from_le_bytes(
                    bytes[body + 8..body + 12].try_into().ok()?,
                ));
            }
            b"fact" if size >= 4 => {
                fact_samples = Some(u32::from_le_bytes(bytes[body..body + 4].try_into().ok()?));
            }
            b"data" => data_size = Some(size as u64),
            _ => {}
        }
        offset = end.checked_add(size & 1)?;
    }
    if let (Some(samples), Some(rate)) = (fact_samples, sample_rate)
        && rate > 0
    {
        return Some(f64::from(samples) / f64::from(rate));
    }
    let bytes_per_second = byte_rate?;
    if bytes_per_second == 0 {
        return None;
    }
    Some(data_size? as f64 / f64::from(bytes_per_second))
}

fn flac_duration(bytes: &[u8]) -> Option<f64> {
    if bytes.get(..4)? != b"fLaC" {
        return None;
    }
    let mut offset = 4usize;
    while offset.checked_add(4)? <= bytes.len() {
        let header = bytes[offset];
        let block_type = header & 0x7f;
        let last = header & 0x80 != 0;
        let size = (usize::from(bytes[offset + 1]) << 16)
            | (usize::from(bytes[offset + 2]) << 8)
            | usize::from(bytes[offset + 3]);
        let body = offset + 4;
        let end = body.checked_add(size)?;
        if end > bytes.len() {
            return None;
        }
        if block_type == 0 && size >= 18 {
            let packed = u64::from_be_bytes(bytes[body + 10..body + 18].try_into().ok()?);
            let sample_rate = (packed >> 44) & 0x000f_ffff;
            let total_samples = packed & 0x0000_000f_ffff_ffff;
            if sample_rate > 0 {
                return Some(total_samples as f64 / sample_rate as f64);
            }
        }
        if last {
            break;
        }
        offset = end;
    }
    None
}

fn ogg_duration(bytes: &[u8]) -> Option<f64> {
    if bytes.get(..4)? != b"OggS" {
        return None;
    }
    let mut offset = 0usize;
    let mut packet = Vec::new();
    let mut sample_rate = None;
    let mut pre_skip = 0u64;
    let mut maximum_granule = None;
    while offset.checked_add(27)? <= bytes.len() {
        if bytes.get(offset..offset + 4)? != b"OggS" || bytes[offset + 4] != 0 {
            return None;
        }
        let granule = u64::from_le_bytes(bytes[offset + 6..offset + 14].try_into().ok()?);
        if granule != u64::MAX {
            maximum_granule =
                Some(maximum_granule.map_or(granule, |value: u64| value.max(granule)));
        }
        let segment_count = usize::from(bytes[offset + 26]);
        let table_start = offset + 27;
        let body_start = table_start.checked_add(segment_count)?;
        if body_start > bytes.len() {
            return None;
        }
        let body_size = bytes[table_start..body_start]
            .iter()
            .map(|value| usize::from(*value))
            .sum::<usize>();
        let page_end = body_start.checked_add(body_size)?;
        if page_end > bytes.len() {
            return None;
        }
        let mut body_offset = body_start;
        for size in &bytes[table_start..body_start] {
            let size = usize::from(*size);
            packet.extend_from_slice(bytes.get(body_offset..body_offset + size)?);
            body_offset += size;
            if size < 255 {
                if sample_rate.is_none() {
                    if packet.starts_with(b"OpusHead") && packet.len() >= 12 {
                        sample_rate = Some(48_000u32);
                        pre_skip = u64::from(u16::from_le_bytes(packet[10..12].try_into().ok()?));
                    } else if packet.starts_with(b"\x01vorbis") && packet.len() >= 16 {
                        sample_rate = Some(u32::from_le_bytes(packet[12..16].try_into().ok()?));
                    }
                }
                packet.clear();
            }
        }
        offset = page_end;
    }
    let rate = u64::from(sample_rate?);
    let samples = maximum_granule?.saturating_sub(pre_skip);
    (rate > 0).then(|| samples as f64 / rate as f64)
}

fn mp4_duration(bytes: &[u8]) -> Option<f64> {
    if bytes.len() < 12 || bytes.get(4..8)? != b"ftyp" {
        return None;
    }
    let mut movie = None;
    let mut media = None;
    scan_mp4_boxes(bytes, 0, &mut movie, &mut media);
    movie.or(media)
}

fn scan_mp4_boxes(
    bytes: &[u8],
    depth: usize,
    movie_duration: &mut Option<f64>,
    media_duration: &mut Option<f64>,
) {
    if depth > 8 {
        return;
    }
    let mut offset = 0usize;
    while offset.saturating_add(8) <= bytes.len() {
        let size32 = u32::from_be_bytes(match bytes[offset..offset + 4].try_into() {
            Ok(value) => value,
            Err(_) => return,
        });
        let box_type = &bytes[offset + 4..offset + 8];
        let (header, size) = if size32 == 1 {
            if offset.saturating_add(16) > bytes.len() {
                return;
            }
            let size = u64::from_be_bytes(match bytes[offset + 8..offset + 16].try_into() {
                Ok(value) => value,
                Err(_) => return,
            });
            let Ok(size) = usize::try_from(size) else {
                return;
            };
            (16usize, size)
        } else if size32 == 0 {
            (8usize, bytes.len() - offset)
        } else {
            (8usize, size32 as usize)
        };
        if size < header || offset.saturating_add(size) > bytes.len() {
            return;
        }
        let content = &bytes[offset + header..offset + size];
        match box_type {
            b"mvhd" => *movie_duration = parse_mp4_time_header(content),
            b"mdhd" if media_duration.is_none() => *media_duration = parse_mp4_time_header(content),
            b"moov" | b"trak" | b"mdia" => {
                scan_mp4_boxes(content, depth + 1, movie_duration, media_duration)
            }
            _ => {}
        }
        offset += size;
    }
}

fn parse_mp4_time_header(bytes: &[u8]) -> Option<f64> {
    let version = *bytes.first()?;
    let (timescale, duration) = if version == 1 {
        if bytes.len() < 32 {
            return None;
        }
        (
            u32::from_be_bytes(bytes[20..24].try_into().ok()?),
            u64::from_be_bytes(bytes[24..32].try_into().ok()?),
        )
    } else {
        if bytes.len() < 20 {
            return None;
        }
        (
            u32::from_be_bytes(bytes[12..16].try_into().ok()?),
            u64::from(u32::from_be_bytes(bytes[16..20].try_into().ok()?)),
        )
    };
    (timescale > 0 && duration != u64::MAX && duration != u64::from(u32::MAX))
        .then(|| duration as f64 / f64::from(timescale))
}

fn webm_duration(bytes: &[u8]) -> Option<f64> {
    if bytes.get(..4)? != [0x1a, 0x45, 0xdf, 0xa3] {
        return None;
    }
    let mut timecode_scale = 1_000_000u64;
    let mut duration = None;
    let mut offset = 0usize;
    while offset < bytes.len() {
        if bytes.get(offset..offset + 3) == Some(&[0x2a, 0xd7, 0xb1]) {
            if let Some((size, body)) = ebml_payload(bytes, offset + 3)
                && (1..=8).contains(&size)
            {
                timecode_scale = read_be_uint(bytes.get(body..body + size)?)?;
            }
        } else if bytes.get(offset..offset + 2) == Some(&[0x44, 0x89])
            && let Some((size, body)) = ebml_payload(bytes, offset + 2)
        {
            duration = match size {
                4 => Some(f64::from(f32::from_bits(u32::from_be_bytes(
                    bytes.get(body..body + 4)?.try_into().ok()?,
                )))),
                8 => Some(f64::from_bits(u64::from_be_bytes(
                    bytes.get(body..body + 8)?.try_into().ok()?,
                ))),
                _ => duration,
            };
        }
        offset += 1;
    }
    Some(duration? * timecode_scale as f64 / 1_000_000_000.0)
}

fn ebml_payload(bytes: &[u8], offset: usize) -> Option<(usize, usize)> {
    let first = *bytes.get(offset)?;
    let length = usize::try_from(first.leading_zeros()).ok()? + 1;
    if length > 8 || offset.checked_add(length)? > bytes.len() {
        return None;
    }
    let mut value = u64::from(first & (0xff >> length));
    for byte in bytes.get(offset + 1..offset + length)? {
        value = (value << 8) | u64::from(*byte);
    }
    let size = usize::try_from(value).ok()?;
    let body = offset + length;
    body.checked_add(size)
        .filter(|end| *end <= bytes.len())
        .map(|_| (size, body))
}

fn read_be_uint(bytes: &[u8]) -> Option<u64> {
    if bytes.len() > 8 {
        return None;
    }
    Some(
        bytes
            .iter()
            .fold(0u64, |value, byte| (value << 8) | u64::from(*byte)),
    )
}

fn mp3_duration(bytes: &[u8]) -> Option<f64> {
    let mut offset = id3v2_end(bytes).unwrap_or(0);
    let mut duration = 0.0;
    let mut frames = 0usize;
    while offset.checked_add(4)? <= bytes.len() {
        let Some(frame) = mp3_frame(&bytes[offset..]) else {
            if frames == 0 {
                offset += 1;
                continue;
            }
            break;
        };
        if offset.checked_add(frame.length)? > bytes.len() {
            break;
        }
        duration += f64::from(frame.samples) / f64::from(frame.sample_rate);
        frames += 1;
        offset += frame.length;
    }
    (frames > 0).then_some(duration)
}

struct Mp3Frame {
    length: usize,
    samples: u32,
    sample_rate: u32,
}

fn mp3_frame(bytes: &[u8]) -> Option<Mp3Frame> {
    let header = u32::from_be_bytes(bytes.get(..4)?.try_into().ok()?);
    if header & 0xffe0_0000 != 0xffe0_0000 {
        return None;
    }
    let version = (header >> 19) & 0x3;
    let layer = (header >> 17) & 0x3;
    let bitrate_index = usize::try_from((header >> 12) & 0xf).ok()?;
    let sample_index = usize::try_from((header >> 10) & 0x3).ok()?;
    if version == 1 || layer == 0 || bitrate_index == 0 || bitrate_index == 15 || sample_index == 3
    {
        return None;
    }
    const MPEG1_LAYER1: [u32; 14] = [
        32, 64, 96, 128, 160, 192, 224, 256, 288, 320, 352, 384, 416, 448,
    ];
    const MPEG1_LAYER2: [u32; 14] = [
        32, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320, 384,
    ];
    const MPEG1_LAYER3: [u32; 14] = [
        32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320,
    ];
    const MPEG2_LAYER1: [u32; 14] = [
        32, 48, 56, 64, 80, 96, 112, 128, 144, 160, 176, 192, 224, 256,
    ];
    const MPEG2_LAYER23: [u32; 14] = [8, 16, 24, 32, 40, 48, 56, 64, 80, 96, 112, 128, 144, 160];
    let table = match (version == 3, layer) {
        (true, 3) => &MPEG1_LAYER1,
        (true, 2) => &MPEG1_LAYER2,
        (true, 1) => &MPEG1_LAYER3,
        (false, 3) => &MPEG2_LAYER1,
        (false, _) => &MPEG2_LAYER23,
        _ => return None,
    };
    let bitrate = table[bitrate_index - 1] * 1_000;
    let base_rate = [44_100u32, 48_000, 32_000][sample_index];
    let sample_rate = match version {
        3 => base_rate,
        2 => base_rate / 2,
        0 => base_rate / 4,
        _ => return None,
    };
    let padding = (header >> 9) & 1;
    let (length, samples) = match layer {
        3 => (((12 * bitrate / sample_rate) + padding) * 4, 384),
        2 => ((144 * bitrate / sample_rate) + padding, 1_152),
        1 if version == 3 => ((144 * bitrate / sample_rate) + padding, 1_152),
        1 => ((72 * bitrate / sample_rate) + padding, 576),
        _ => return None,
    };
    Some(Mp3Frame {
        length: usize::try_from(length).ok()?,
        samples,
        sample_rate,
    })
}

fn id3v2_end(bytes: &[u8]) -> Option<usize> {
    if bytes.get(..3)? != b"ID3" || bytes.len() < 10 {
        return None;
    }
    let size_bytes = bytes.get(6..10)?;
    if size_bytes.iter().any(|value| value & 0x80 != 0) {
        return None;
    }
    let size = size_bytes
        .iter()
        .fold(0usize, |value, byte| (value << 7) | usize::from(*byte));
    let footer = (bytes[5] & 0x10 != 0) as usize * 10;
    10usize.checked_add(size)?.checked_add(footer)
}

fn adts_duration(bytes: &[u8]) -> Option<f64> {
    const SAMPLE_RATES: [u32; 13] = [
        96_000, 88_200, 64_000, 48_000, 44_100, 32_000, 24_000, 22_050, 16_000, 12_000, 11_025,
        8_000, 7_350,
    ];
    let mut offset = 0usize;
    let mut duration = 0.0;
    let mut frames = 0usize;
    while offset.checked_add(7)? <= bytes.len() {
        if bytes[offset] != 0xff || bytes[offset + 1] & 0xf6 != 0xf0 {
            if frames == 0 {
                offset += 1;
                continue;
            }
            break;
        }
        let rate_index = usize::from((bytes[offset + 2] >> 2) & 0x0f);
        let sample_rate = *SAMPLE_RATES.get(rate_index)?;
        let frame_length = (usize::from(bytes[offset + 3] & 0x03) << 11)
            | (usize::from(bytes[offset + 4]) << 3)
            | usize::from(bytes[offset + 5] >> 5);
        let header_length = if bytes[offset + 1] & 1 == 0 { 9 } else { 7 };
        if frame_length < header_length || offset.checked_add(frame_length)? > bytes.len() {
            break;
        }
        let blocks = u32::from(bytes[offset + 6] & 0x03) + 1;
        duration += f64::from(1_024 * blocks) / f64::from(sample_rate);
        frames += 1;
        offset += frame_length;
    }
    (frames > 0).then_some(duration)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_pcm_wav_duration_without_decoding_samples() {
        let sample_rate = 1_000u32;
        let sample_count = 1_612u32;
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
        let metadata = parse(&bytes, "audio/wav").expect("WAV metadata");
        assert!((metadata.duration - 1.612).abs() < f64::EPSILON);
    }

    #[test]
    fn reads_flac_streaminfo_duration() {
        let sample_rate = 48_000u64;
        let total_samples = 77_376u64;
        let packed = (sample_rate << 44) | (1 << 41) | (15 << 36) | total_samples;
        let mut bytes = b"fLaC".to_vec();
        bytes.extend_from_slice(&[0x80, 0, 0, 34]);
        bytes.extend_from_slice(&[0; 10]);
        bytes.extend_from_slice(&packed.to_be_bytes());
        bytes.extend_from_slice(&[0; 16]);
        let metadata = parse(&bytes, "audio/flac").expect("FLAC metadata");
        assert!((metadata.duration - 1.612).abs() < 1e-12);
    }

    #[test]
    fn reads_ogg_opus_granule_duration() {
        fn page(granule: u64, packet: &[u8], header_type: u8) -> Vec<u8> {
            let mut page = b"OggS".to_vec();
            page.extend_from_slice(&[0, header_type]);
            page.extend_from_slice(&granule.to_le_bytes());
            page.extend_from_slice(&1u32.to_le_bytes());
            page.extend_from_slice(&0u32.to_le_bytes());
            page.extend_from_slice(&0u32.to_le_bytes());
            page.push((!packet.is_empty()) as u8);
            if !packet.is_empty() {
                page.push(packet.len() as u8);
                page.extend_from_slice(packet);
            }
            page
        }
        let mut opus_head = b"OpusHead".to_vec();
        opus_head.extend_from_slice(&[1, 2]);
        opus_head.extend_from_slice(&312u16.to_le_bytes());
        opus_head.extend_from_slice(&48_000u32.to_le_bytes());
        opus_head.extend_from_slice(&[0, 0, 0]);
        let mut bytes = page(0, &opus_head, 2);
        bytes.extend_from_slice(&page(77_688, &[], 4));
        let metadata = parse(&bytes, "audio/ogg").expect("Ogg Opus metadata");
        assert!((metadata.duration - 1.612).abs() < 1e-12);
    }

    #[test]
    fn reads_mp4_movie_header_duration() {
        fn mp4_box(kind: &[u8; 4], body: &[u8]) -> Vec<u8> {
            let mut output = Vec::new();
            output.extend_from_slice(&(8u32 + body.len() as u32).to_be_bytes());
            output.extend_from_slice(kind);
            output.extend_from_slice(body);
            output
        }
        let mut bytes = mp4_box(b"ftyp", b"isom\0\0\0\0");
        let mut mvhd = vec![0; 12];
        mvhd.extend_from_slice(&1_000u32.to_be_bytes());
        mvhd.extend_from_slice(&1_612u32.to_be_bytes());
        let mvhd = mp4_box(b"mvhd", &mvhd);
        bytes.extend_from_slice(&mp4_box(b"moov", &mvhd));
        let metadata = parse(&bytes, "audio/mp4").expect("MP4 metadata");
        assert!((metadata.duration - 1.612).abs() < 1e-12);
    }

    #[test]
    fn reads_webm_info_duration() {
        let mut bytes = vec![0x1a, 0x45, 0xdf, 0xa3, 0x80];
        bytes.extend_from_slice(&[0x2a, 0xd7, 0xb1, 0x83, 0x0f, 0x42, 0x40]);
        bytes.extend_from_slice(&[0x44, 0x89, 0x88]);
        bytes.extend_from_slice(&1_612f64.to_bits().to_be_bytes());
        let metadata = parse(&bytes, "audio/webm").expect("WebM metadata");
        assert!((metadata.duration - 1.612).abs() < 1e-12);
    }

    #[test]
    fn reads_mp3_frame_headers_duration() {
        let frame_length = 144 * 128_000 / 44_100;
        let mut bytes = vec![0; frame_length * 2];
        bytes[..4].copy_from_slice(&[0xff, 0xfb, 0x90, 0x00]);
        bytes[frame_length..frame_length + 4].copy_from_slice(&[0xff, 0xfb, 0x90, 0x00]);
        let metadata = parse(&bytes, "audio/mpeg").expect("MP3 metadata");
        assert!((metadata.duration - (2.0 * 1_152.0 / 44_100.0)).abs() < 1e-12);
    }

    #[test]
    fn reads_aac_adts_frame_headers_duration() {
        let bytes = [0xff, 0xf1, 0x50, 0x80, 0x00, 0xff, 0xfc];
        let metadata = parse(&bytes, "audio/aac").expect("ADTS metadata");
        assert!((metadata.duration - (1_024.0 / 44_100.0)).abs() < 1e-12);
    }
}
