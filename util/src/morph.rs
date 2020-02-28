///
/// The morphological 'opening' operator for 1d. Used to fill-in
/// small gaps in areas of high activity
/// 
pub fn morph_opening(input: &[bool], radius: usize) -> Vec<bool> {
    let eroded = morph_erosion(input, radius);
    let dilated = morph_dilation(&eroded, radius);
    dilated
}

///
/// The morphological 'closing' operator for 1d. Used to remove
/// small voice segments from areas of low activity
/// 
pub fn morph_closing(input: &[bool], radius: usize) -> Vec<bool> {
    let dilated = morph_dilation(input, radius);
    let eroded = morph_erosion(&dilated, radius);
    eroded
}

///
/// The morphological 'dilate' operator for 1d
/// 
pub fn morph_dilation(input: &[bool], radius: usize) -> Vec<bool> {
    let mut output = input.to_vec();
    let (start, end) = (radius, input.len() - radius);
    for i in start .. end {
        if !input[i] {
            for j in 1 ..= radius {
                if input[i-j] || input[i+j] {
                    output[i] = true;
                    break;
                }
            }
        }
    }
    output
}

///
/// The morphological 'erode' operator for 1d
/// 
pub fn morph_erosion(input: &[bool], radius: usize) -> Vec<bool> {
    let mut output = input.to_vec();
    let (start, end) = (radius, input.len() - radius);
    for i in start .. end {
        if input[i] {
            for j in 1 ..= radius {
                if !(input[i-j] && input[i+j]) {
                    output[i] = false;
                    break;
                }
            }
        }
    }
    output
}
