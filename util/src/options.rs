///
/// Options governing the synchronization process
/// 
/// Most of these options are passed directly to `alass`. See the official `alass` repository for
/// details: https://github.com/kaegi/alass
/// 
/// * `interval`: The smallest unit of time recognized by `alass`. Smaller numbers make the alignment
///    more accurate, larger numbers make alignment faster. (millis)
/// 
/// * `split_mode`: When true, `alass` will attempt alignment assuming the presence of commercial breaks
///    or added/removed scenes. Disabling `split_mode` can make syncing faster but will only correct
///    subtitles whose misalignment is the result of a constant shift.
/// 
/// * `split_penalty`: Determines how eager the algorithm is to avoid splitting of the subtitles. A
///    value of 1000 means that all lines will be shifted by the same offset, while 0.01 will produce
///    MANY segments with different offsets. Values from 1 to 20 are the most reasonable.
/// 
/// * `speed_optimization`: Greatly speeds up synchronization by sacrificing accuracy.
/// 
/// * `framerate_correction`: Whether to attempt correction of mismatched framerates.
/// 
pub struct SyncOptions {
    pub interval: i64,
    pub split_mode: bool,
    pub split_penalty: f64,
    pub speed_optimization: Option<f64>,
    pub framerate_correction: bool
}

impl Default for SyncOptions {
    fn default() -> Self {
        SyncOptions {
            interval: 60,
            split_mode: true,
            split_penalty: 7.0,
            speed_optimization: Some(1.0),
            framerate_correction: false
        }
    }
}
