
use crate::morph::{morph_opening, morph_closing};

///
/// Voice activity as computed by either WebRtcVad or Silero
/// 
pub struct VoiceActivity {
    pub data: Vec<bool>,
    pub chunk_millis: usize
}

impl VoiceActivity {

    ///
    /// Cleans voice-activity data (EXPERIMENTAL)
    /// 
    /// This operation successively employs the mathematical morphological 'erosion'
    /// and 'dilation` operators to clean the output of the voice-activity detector.
    /// The result is a clone of the original `VoiceActivity` instance having
    /// cleaner/fewer timespans.
    /// 
    /// The `opening_radius` and `closing_radius` parameters represent the kernel radii
    /// of the mathematical morphological operators. Each radius determines a window
    /// of size `(2r+1)*CHUNK_MILLIS` milliseconds. Any errant spans smaller than this
    /// window will be removed and any gaps larger than this window will be filled.
    /// 
    pub fn clean(self: &Self, opening_radius: usize, closing_radius: usize) -> Self {
    
        // Clone voice-activity buffer and add padding
        fn pad(data: &[bool], radius: usize) -> Vec<bool> {
            let orig_len = data.len();
            let clone_len = orig_len + radius * 2;
            let mut clone = vec![false; clone_len];
            clone.truncate(radius);
            clone.extend_from_slice(&data);
            clone.resize(clone_len, false);
            clone
        }
    
        // Remove padding
        fn unpad(data: &[bool], radius: usize) -> Vec<bool> {
            data[radius .. data.len()-radius].to_vec()
        }

        let data = self.data.to_owned();

        // Perform morphological 'closing' operation to fill gaps
        let data = if opening_radius > 0 {
            let opening_clone = pad(&data, opening_radius);
            unpad(&morph_opening(&opening_clone, opening_radius), opening_radius)
        } else {
            data
        };

        // Perform morphological 'opening' operation to remove noise
        let data = if closing_radius > 0 {
            let closing_clone = pad(&data, closing_radius);
            unpad(&morph_closing(&closing_clone, closing_radius), closing_radius)
        } else {
            data
        };

        VoiceActivity { data, chunk_millis: self.chunk_millis }
    }
}
