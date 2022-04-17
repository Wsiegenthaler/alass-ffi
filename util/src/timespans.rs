use std::fmt;
use std::fs::File;
use std::path::Path;
use std::io;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter, Cursor};
use std::io::ErrorKind::NotFound;
use std::error::Error;
use std::iter::once;
use std::convert::{TryFrom, TryInto};
use std::cmp::{min, max};

use alass_core::TimeSpan;

use subparse::SubtitleFile;
use subparse::timetypes::{TimeSpan as SubTimeSpan, TimePoint as SubTimePoint};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use crate::{SyncError, VoiceActivity};
use crate::util::to_alass_timepoint;

use TimeSpansLoadError::*;
use TimeSpansSaveError::*;

///
/// `TimeSpan` buffer to be used as reference by the synchronization process
/// 
/// May be generated from an `AudioSink` that's been processed for voice-
/// activity, extracted from a subtitle file whose timing is known to be good,
/// or populated manually by some other process (e.g. extracing embedded subs
/// from video file). `TimeSpans` also provides functionality for reading and
/// writing raw timespan data to/from disk which is useful for caching.
/// 
pub struct TimeSpans(pub Vec<SubTimeSpan>);

impl TimeSpans {

    ///
    /// Appends a timespan
    /// 
    pub fn push(self: &mut Self, span: SubTimeSpan) {
        self.0.push(span)
    }

    ///
    /// Saves raw timespan data to disk
    /// 
    pub fn save(self: &Self, filename: &str) -> Result<(), TimeSpansSaveError> {
        let file = File::create(Path::new(filename))
            .map_err(|cause| WriteError { cause })?;
        let bytes: Vec<u8> = self.try_into()
            .map_err(|cause| SerializeError { cause })?;
        let _ = BufWriter::new(file)
            .write(bytes.as_slice())
            .map_err(|cause| WriteError { cause })?;
        Ok(())
    }

    ///
    /// Loads raw timespan data from disk
    /// 
    pub fn load(filename: &str) -> Result<Self, TimeSpansLoadError> {
        let file = File::open(Path::new(filename))
            .map_err(|cause| match cause.kind() {
                NotFound => FileNotFound { path: filename.to_string() },
                _ => ReadError { cause, path: filename.to_string() }
            })?;
        let bytes = &mut vec![];
        BufReader::new(file)
            .read_to_end(bytes)
            .map_err(|cause| ReadError { cause, path: filename.to_string() })?;
        let spans: TimeSpans = Self::try_from(bytes.as_slice())
            .map_err(|cause| DeserializeError { cause })?;
        Ok(spans)
    }

    ///
    /// Create `TimeSpans` instance from subtitle file
    /// 
    pub fn from_sub_file(sub_file: &SubtitleFile) -> Result<Self, SyncError> {
        let entries = sub_file.get_subtitle_entries().expect("Unable to read subtitle entries");
        let spans = TimeSpans(entries.into_iter()
            .map(|subentry| subentry.timespan)
            .map(|span: SubTimeSpan| SubTimeSpan::new(min(span.start, span.end), max(span.start, span.end)))
            .collect());
        Ok(spans)
    }

    ///
    /// Convert `TimeSpan`s from `subparse` to `alass_core` representation
    /// 
    pub fn to_alass_timespans(self: &Self, interval: i64) -> Vec<TimeSpan> {
        self.0.iter().cloned()
            .map(|timespan| {
                TimeSpan::new_safe(
                    to_alass_timepoint(timespan.start, interval),
                    to_alass_timepoint(timespan.end, interval),
                )
            }).collect()
    }
}

///
/// Allow for iteration over internal `subparse::TimeSpan` elements
/// 
impl IntoIterator for TimeSpans {
    type Item = SubTimeSpan;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

///
/// Serializes `TimeSpans` instance to raw bytes
/// 
impl<'a> TryInto<Vec<u8>> for &'a TimeSpans {
    type Error = io::Error;

    fn try_into(self: &'a TimeSpans) -> Result<Vec<u8>, Self::Error> {
        let mut bytes = vec![];
        bytes.write_u32::<LittleEndian>(self.0.len() as u32)?;
        for s in self.0.iter() {
            bytes.write_i64::<LittleEndian>(s.start.msecs())?;
            bytes.write_i64::<LittleEndian>(s.end.msecs())?;
        }
        Ok(bytes)
    }
}

///
/// Deserializes `TimeSpans` instance from raw bytes
/// 
impl TryFrom<&[u8]> for TimeSpans {
    type Error = io::Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let mut rdr = Cursor::new(bytes);
        let count = rdr.read_u32::<LittleEndian>()?;
        let spans: Result<Vec<SubTimeSpan>, io::Error> = (0..count).map(|_| {
            let start = rdr.read_i64::<LittleEndian>()?;
            let end = rdr.read_i64::<LittleEndian>()?;
            Ok(SubTimeSpan {
                start: SubTimePoint::from_msecs(start),
                end: SubTimePoint::from_msecs(end)
            })
        }).collect();
        Ok(TimeSpans(spans?))
    }
}

///
/// Analyze vector of voice-activity data and produce `TimeSpans`
/// 
impl From<&VoiceActivity> for TimeSpans {
    fn from(activity: &VoiceActivity) -> TimeSpans {
        let timespans: Vec<SubTimeSpan> =
            once(&false).chain(activity.data.iter().chain(once(&false)))
                .collect::<Vec<&bool>>()
                .windows(2)
                .enumerate()
                .filter_map(|(i, v)| if *v[0] != *v[1] { Some(i as i64) } else { None })
                .map(|t| SubTimePoint::from_msecs(t * activity.chunk_millis as i64))
                .collect::<Vec<SubTimePoint>>()
                .chunks(2)
                .map(|s| SubTimeSpan::new(s[0], s[1]))
                .collect();
        TimeSpans(timespans)
    }
}

///
/// Represents an error reading raw timespan data from disk
/// 
#[derive(Debug)]
pub enum TimeSpansLoadError {
    FileNotFound { path: String },
    ReadError { cause: io::Error, path: String },
    DeserializeError { cause: io::Error },
}

impl Error for TimeSpansLoadError {}

impl fmt::Display for TimeSpansLoadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FileNotFound { path } => write!(f, "Span file does not exist (path='{}')", path),
            ReadError { cause, path } => write!(f, "Error reading timespans from disk (msg='{}', path='{}')", cause, path),
            DeserializeError { cause } => write!(f, "Error parsing timespans from bytes (msg='{}')", cause)
        }
    }
}

///
/// Represents an error writing raw timespan data to disk
/// 
#[derive(Debug)]
pub enum TimeSpansSaveError {
    SerializeError { cause: io::Error },
    WriteError { cause: io::Error }
}

impl Error for TimeSpansSaveError {}

impl fmt::Display for TimeSpansSaveError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SerializeError { cause } => write!(f, "Error serializing timespans to bytes! (msg='{}')", cause),
            WriteError { cause } => write!(f, "Error saving timespan data to disk! (msg='{}')", cause)
        }
    }
}
