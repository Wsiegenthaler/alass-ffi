use std::vec;
use std::io::Cursor;
use std::include_bytes;

use tract_onnx::prelude::*;
use tract_ndarray::*;


const PROB_THRESHOLD: f32 = 0.7;

type RunnableOnnxModel = SimplePlan<TypedFact, Box<dyn TypedOp>, Graph<TypedFact, Box<dyn TypedOp>>>;

pub struct Vad {
    model: RunnableOnnxModel,
    h: Tensor,
    c: Tensor,
    pub chunk_size: usize
}

impl Vad {

    pub fn new() -> Result<Self, String> {
        let chunk_size = (Self::expected_chunk_millis() as f32 / 1000f32 * Self::expected_sample_rate() as f32) as usize;

        match Self::model(chunk_size) {
            Ok(model) => {
                // Model state
                let h: Tensor = Array3::from_shape_fn((2, 1, 64), |(_, _, _)| 0f32).into();
                let c: Tensor = Array3::from_shape_fn((2, 1, 64), |(_, _, _)| 0f32).into();
      
                Ok(Vad { model, h, c, chunk_size })
            },
            Err(err) => Err(format!("Unable to create silaro-vad onnx graph! ({})", err))
        }
    }

    fn model(chunk_size: usize) -> TractResult<RunnableOnnxModel> {
        let model: RunnableOnnxModel = tract_onnx::onnx()
            .model_for_read(&mut Cursor::new(include_bytes!("silero_net.onnx")))?
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
        Ok(PROB_THRESHOLD <= output[[0, 1, 0]])
    }

    pub fn is_voice_segment(&mut self, chunk: &[i16]) -> Result<bool, String> {
        match self.process_chunk(chunk) {
            Ok(is_voice) => Ok(is_voice),
            Err(err) => Err(format!("The silero-vad module encountered an error while processing samples for voice activity. ({})", err))
        }
    }

    pub fn expected_sample_rate() -> usize {
        16000
    }

    pub fn expected_chunk_millis() -> usize {
        96
    }
}
