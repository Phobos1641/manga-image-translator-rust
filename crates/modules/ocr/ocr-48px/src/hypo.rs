use ndarray::Array3;

#[derive(Clone)]
pub struct Hypothesis {
    pub memory_idx: i64,
    end_tok: i64,
    pub cached_activations: Vec<Array3<f32>>,
    pub out_idx: Vec<i64>,
    out_logprobs: Vec<f32>,
    length: usize,
}

impl Hypothesis {
    pub fn new(
        num_layers: u8,
        embd_dim: usize,
        memory_idx: i64,
        start_tok: i64,
        end_tok: i64,
    ) -> Self {
        Self {
            memory_idx,
            end_tok,
            // 1, L, E
            // TODO: store as tensor?
            cached_activations: vec![Array3::zeros((1, 0, embd_dim)); num_layers as usize + 1],
            out_idx: vec![start_tok],
            out_logprobs: vec![0.0],
            length: 0,
        }
    }

    pub fn seq_end(&self) -> bool {
        self.out_idx.last() == Some(&self.end_tok)
    }

    pub fn logprob(&self) -> f32 {
        self.out_logprobs.iter().sum::<f32>() / self.out_logprobs.len() as f32
    }

    pub fn sort_key(&self) -> f32 {
        -self.logprob()
    }

    pub fn prob(&self) -> f32 {
        self.logprob().exp()
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn output_owned(mut self) -> Array3<f32> {
        self.cached_activations.pop().unwrap()
    }

    pub fn extend(mut self, idx: i64, logprob: f32) -> Self {
        self.length += 1;
        //TODO: clone tensor
        self.out_idx.push(idx);
        self.out_logprobs.push(logprob);
        self
    }
}
