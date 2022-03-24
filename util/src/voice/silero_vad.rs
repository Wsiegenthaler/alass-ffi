use std::vec;
use std::io::Cursor;
use std::include_bytes;

use tract_onnx::prelude::*;
use tract_ndarray::*;

use crate::voice::VoiceDetector;


type RunnableOnnxModel = SimplePlan<TypedFact, Box<dyn TypedOp>, Graph<TypedFact, Box<dyn TypedOp>>>;

pub struct SileroVad {
    model: RunnableOnnxModel,
    h: Tensor,
    c: Tensor,
    chunk_size: usize,
    threshold: f32
}

impl SileroVad {

    pub fn new(chunk_size: usize, threshold: f32) -> Result<Self, String> {
        match Self::model(chunk_size) {
            Ok(model) => {
                // Model state
                let h: Tensor = Array3::from_shape_fn((2, 1, 64), |(_, _, _)| 0f32).into();
                let c: Tensor = Array3::from_shape_fn((2, 1, 64), |(_, _, _)| 0f32).into();
      
                Ok(SileroVad {
                    model,
                    h,
                    c,
                    chunk_size,
                    threshold
                })
            },
            Err(err) => Err(format!("Unable to create silaro-vad onnx graph! ({})", err))
        }
    }

    fn model(chunk_size: usize) -> TractResult<RunnableOnnxModel> {
        let model: RunnableOnnxModel = tract_onnx::onnx()
            .model_for_read(&mut Cursor::new(include_bytes!("silero_vad.onnx")))?
            .with_input_names(&["input", "h0", "c0"])?
            .with_output_names(&["output", "hn", "cn"])?
            .with_input_fact(0, InferenceFact::dt_shape(f32::datum_type(), tvec!(1, chunk_size)))?
            .with_input_fact(1, InferenceFact::dt_shape(f32::datum_type(), tvec!(2, 1, 64)))?
            .with_input_fact(2, InferenceFact::dt_shape(f32::datum_type(), tvec!(2, 1, 64)))?
            .into_optimized()?
            .into_runnable()?;
        Ok(model)
    }

    fn process_chunk(&mut self, chunk: &[i16]) -> TractResult<bool> {
        let input: Tensor = Array2::from_shape_fn((1, self.chunk_size), |(_, j)| chunk[j] as f32).into();
        let result = &self.model.run(tvec![input, self.h.clone(), self.c.clone()])?;
        
        // Extract results/state
        let output = result[0].to_array_view::<f32>()?;
        let hn = result[1].to_array_view::<f32>()?;
        let cn = result[2].to_array_view::<f32>()?;

        // Save state
        self.h = Array3::from_shape_fn((2, 1, 64), |(i, j, k)| hn[[i, j, k]]).into();
        self.c = Array3::from_shape_fn((2, 1, 64), |(i, j, k)| cn[[i, j, k]]).into();

        // Determine voice activity
        Ok(self.threshold <= output[[0, 1, 0]])
    }

}

impl VoiceDetector for SileroVad {
    fn is_voice_segment(&mut self, chunk: &[i16]) -> Result<bool, String> {
        match self.process_chunk(chunk) {
            Ok(is_voice) => Ok(is_voice),
            Err(err) => Err(format!("The silero-vad module encountered an error while processing samples for voice activity. ({})", err))
        }
    }
}