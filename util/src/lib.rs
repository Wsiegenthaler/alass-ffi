extern crate chardet;

use std::{fmt, io, vec};
use std::fs::File;
use std::path::Path;
use std::error::Error;
use std::io::{Read, Write};

use log::*;

use subparse::*;
use subparse::errors::{Result as SubparseResult, ErrorKind::UpdatingEntriesNotSupported};

use encoding_rs::{Encoding, UTF_8};

use alass_core::*;

mod audiosink;
mod timespans;
mod voice_activity;
mod options;
mod vad;

///
/// Various utility functions for conversions, charset detection, etc.
/// 
pub mod util;

///
/// [Experimental] Mathematical morphological operators for cleaning voice activity data
/// 
pub mod morph;

pub use audiosink::*;
pub use timespans::*;
pub use voice_activity::VoiceActivity;
pub use options::SyncOptions;
use util::*;

use SyncError::*;

/// Standard framerates for framerate correction
pub const FRAMERATES: &[f64] = &[ 23.976023976024, 24.0, 25.0, 29.97002997003, 30.0, 50.0, 59.9400599400599, 60.0 ];

/// Default value used to interpret MicroDVD '.sub' files which use frame ids instead of timestamps
pub const DEFAULT_MICRODVD_FPS: f64 = 30.0;

///
/// Reads and parses an input subtitle file, synchronizes it with the given reference timespans using the
/// `alass` crate, and writes the corrected subtitles back to disk.
/// 
/// * `sub_path_in`: Path to the incorrect subtitle file.
/// 
/// * `sub_path_out`: Path to which the synchronized subtitle file shall
///    be written (must include filename).
/// 
/// * `ref_spans`: Reference timespans to use for alignment.
/// 
/// * `ref_fps`: Framerate of the reference video file (used for framerate correction).
/// 
/// * `sub_encoding`: The IANA charset encoding of the subtitle file. If 'auto' is
///    given (or if not specified), an attempt is made to guess the correct encoding
///    based on the contents of the file.
/// 
/// * `options`: Parameters governing various aspects of the synchronization process. See
///    `SyncOptions` or `alass` documentation for details.
/// 
pub fn sync(
    sub_path_in: &str,
    sub_path_out: &str,
    ref_spans: &TimeSpans,
    ref_fps: f64,
    sub_encoding: Option<String>,
    opt: &SyncOptions
) -> Result<(), SyncError> {

    let (mut sub_file_in, _) = open_sub_file(sub_path_in, sub_encoding)?;
    let sub_spans_in = TimeSpans::from_sub_file(&sub_file_in)?;

    let sub_spans: Vec<TimeSpan> = sub_spans_in.to_alass_timespans(opt.interval);
    let ref_spans: Vec<TimeSpan> = ref_spans.to_alass_timespans(opt.interval);

    // Framerate correction
    let (fps_factor, sub_spans): (f64, Vec<TimeSpan>) = 
        if opt.framerate_correction {
            let (_, fr) = guess_fps_ratio(&ref_spans, &sub_spans, ref_fps);
            if fr.ratio - 1.0 < std::f64::EPSILON {
                info!("detected framerate = {:.3} (reference_framerate = {:.3})", fr.fps, ref_fps)
            };
            let scaled_spans = sub_spans.into_iter().map(|x| x.scaled(fr.ratio)).collect();
            (fr.ratio, scaled_spans)
        } else {
            (1.0, sub_spans)
        };

    // Align spans with reference
    let deltas = 
        if !opt.split_mode {
            let inc_span_cnt = sub_spans.len();
            let (delta, _) = align_nosplit(&ref_spans, &sub_spans, standard_scoring, NoProgressInfo {});
            info!("no split mode: shifting subtitles by {}ms", delta * opt.interval);
            vec::from_elem(delta, inc_span_cnt)
        } else {
            let (deltas, _) = align(&ref_spans, &sub_spans, opt.split_penalty, opt.speed_optimization, standard_scoring, NoProgressInfo {});
            info!("split mode: shifting first subtitle by {}ms and last by {}ms", delta_str(deltas.first(), opt.interval), delta_str(deltas.last(), opt.interval));
            deltas
        };
    
    // Generate corrected subtitle entries
    let corrected_entries: Vec<SubtitleEntry> = sub_spans_in.into_iter()
        .zip(to_subparse_deltas(&deltas, opt.interval))
        .map(|(timespan, delta)| scaled_timespan(timespan, fps_factor) + delta)
        .map(SubtitleEntry::from)
        .collect();

    // Update subtitle file
    sub_file_in.update_subtitle_entries(&corrected_entries)
        .map_err(|e| match e.kind() {
            UpdatingEntriesNotSupported { format } => UnsupportedFormat { format: Some(format), path: sub_path_in.to_string() },
            e => InternalError { msg: format!("Error while updating subtitle entries ({})", e) }
        })?;

    // Write corrected file to disk
    save_sub_file(sub_path_out, &sub_file_in)?;

    Ok(())
}

///
/// Ensure that the format of a given subtitle file supports syncing
/// 
pub fn is_format_supported(sub_path: &str) -> Result<(), SyncError> {
    let data = get_file_bytes(sub_path)?;
    let extension = Path::new(sub_path).extension();
    let format = get_subtitle_format_err(extension, &data)
        .map_err(|e| match e.kind() {
            UpdatingEntriesNotSupported { format } =>
                UnsupportedFormat { path: sub_path.to_string(), format: Some(format) },
            e => InternalError { msg: format!("Error while determining subtitle format ({})", e) }
        })?;

    // Ensure extension matches subtitle format
    if !is_valid_extension_for_subtitle_format(extension, format) {
        return Err(UnsupportedFormat { path: sub_path.to_string(), format: Some(format) });
    }

    // Ensure format is known to support updating entries (this may need to change with updates to subparse)
    match format {
        SubtitleFormat::VobSubSub =>
            Err(UnsupportedFormat { path: sub_path.to_string(), format: Some(format) }),
        _ => Ok(())
    }
}

///
/// Reads and parses subtitle file from disk
/// 
pub fn open_sub_file(path: &str, sub_encoding: Option<String>) -> Result<(SubtitleFile, SubtitleFormat), SyncError> {

    // Read contents of file and determine format
    let data = get_file_bytes(path)?;
    let format = get_subtitle_format_err(Path::new(path).extension(), &data)
        .map_err(|_| UnsupportedFormat { path: path.to_string(), format: None })?;

    // Parse
    let sub_file = parse_sub_file(&data, format, sub_encoding)
        .map_err(|_| ParseError { path: path.to_string() })?;

    Ok((sub_file, format))
}

///
/// Parses raw subtitle content with either user-specified or auto-detected encoding. If parsing
/// fails with the user-specified encoding, will detect and try again.
/// 
pub fn parse_sub_file(data: &[u8], format: SubtitleFormat, sub_encoding: Option<String>) -> SubparseResult<SubtitleFile> {
    let parse_bytes = |encoding: &'static Encoding| -> SubparseResult<SubtitleFile> {
        subparse::parse_bytes(format, &data, Some(encoding), DEFAULT_MICRODVD_FPS)
    };
    match lookup_encoding(sub_encoding) {
        Some(user_encoding) => 
            parse_bytes(user_encoding).or_else(|_| {
                let detected_encoding = detect_encoding_or(&data, UTF_8);
                error!("Error parsing subtitles as '{}', trying '{}'...", user_encoding.name(), detected_encoding.name());
                parse_bytes(detected_encoding)
            }),
        None => parse_bytes(detect_encoding_or(&data, UTF_8))
    }
}

///
/// Writes a subtitle file to disk at the given path
/// 
pub fn save_sub_file(path: &str, sub_file: &SubtitleFile) -> Result<(), SyncError> {
    let sub_data = sub_file.to_data()
        .map_err(|e| SerializeError { path: path.to_string(), cause: e })?;
    let mut file = File::create(Path::new(&path))
        .map_err(|e| WriteError { path: path.to_string(), cause: e })?;
    file.write_all(&sub_data)
        .map_err(|e| WriteError { path: path.to_string(), cause: e })?;
    Ok(())
}

///
/// Reads a file in it's entirety and produces it's bytes
/// 
fn get_file_bytes(path: &str) -> Result<Vec<u8>, SyncError> {
    let mut buf = Vec::new();
    File::open(Path::new(path))
        .and_then(|mut f| f.read_to_end(&mut buf))
        .map_err(|e| {
            let path = path.to_string();
            match e.kind() {
                io::ErrorKind::NotFound => DoesNotExist { path },
                io::ErrorKind::PermissionDenied => PermissionDenied { path },
                _ => ReadError { cause: e, path  }
            }
        })?;
    Ok(buf)
}

///
/// A framerate and it's ratio to the reference framerate
/// 
struct FramerateRatio { pub fps: f64, pub ratio: f64 }

///
/// Aligns timespans using several candidate framerates and returns the best
/// 
fn guess_fps_ratio(ref_spans: &[TimeSpan], inc_spans: &[TimeSpan], ref_fps: f64) -> (f64, FramerateRatio) {
    FRAMERATES.iter()
        .map(|&fps| FramerateRatio { fps, ratio: fps / ref_fps })
        .map(|fr| {
            let candidate_spans: Vec<TimeSpan> = inc_spans.iter().map(|ts| ts.scaled(fr.ratio)).collect();
            let (delta, score) = align_nosplit(ref_spans, &candidate_spans, overlap_scoring, NoProgressHandler);
            debug!("checking framerate {:.4}fps (score: {:.4}, delta: {}ms)", fr.fps, score, delta);
            (score, fr)
        })
        .max_by_key(|(score, _)| (score * 1_000_000_000.0) as i64)
        .unwrap()
}

// Progress noop
struct NoProgressInfo {}
impl ProgressHandler for NoProgressInfo {
    fn init(&mut self, _steps: i64) {}
    fn inc(&mut self) {}
    fn finish(&mut self) {}
}

///
/// Represents errors that may occur during syncing
/// 
#[derive(Debug)]
pub enum SyncError {
    UnsupportedFormat { path: String, format: Option<SubtitleFormat> },
    ReadError { path: String, cause: io::Error },
    DoesNotExist { path: String },
    PermissionDenied { path: String },
    ParseError { path: String },
    WriteError { path: String, cause: io::Error },
    SerializeError { path: String, cause: subparse::errors::Error },
    InternalError { msg: String },
}

impl Error for SyncError {}

impl fmt::Display for SyncError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UnsupportedFormat { path, format } => {
                let format_name = format.map(|f| f.get_name()).unwrap_or("N/A");
                write!(f, "Subtitle format not supported (path='{}', format='{}')", path, format_name)
            },
            ReadError { path, cause } => write!(f, "Error reading subtitle file from disk (msg='{}' path='{}')", cause, path),
            DoesNotExist { path } => write!(f, "Subtitle file does not exist (path='{}')", path),
            PermissionDenied { path } => write!(f, "Insufficient privileges to open subtitle file (path='{}')", path),
            ParseError { path } => write!(f, "Error parsing subtitle file (path='{}')", path),
            WriteError { path, cause } => write!(f, "Error writing subtitle data to disk (msg='{}' path='{}')", cause, path),
            SerializeError { path, cause } => write!(f, "Error serializing subtitle data (msg='{}', path='{}')", cause.kind(), path),
            InternalError { msg } => write!(f, "Unknown sync error occurred (msg='{}')", msg)
        }
    }
}
