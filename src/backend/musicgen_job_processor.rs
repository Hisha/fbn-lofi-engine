use std::collections::VecDeque;
use std::sync::Arc;

use crate::backend::audio_generation_backend::JobProcessor;
use crate::musicgen_models::MusicGenModels;

pub struct MusicGenJobProcessor {
    model: Arc<MusicGenModels>,
}

impl MusicGenJobProcessor {
    pub fn new(model: Arc<MusicGenModels>) -> Self {
        Self { model }
    }

    /// Core inference for a single chunk.
    fn generate_chunk(
        &self,
        prompt: &str,
        secs: usize,
        history: Option<&[f32]>,
    ) -> ort::Result<VecDeque<f32>> {
        // Delegate to model for actual inference, optionally using history for continuity
        self.model.generate(prompt, secs, history)
    }

    /// Infinite chunked generation with overlap
pub fn process_infinite(
    &self,
    prompt: &str,
    total_secs: usize,
    chunksize: usize,
    overlap: usize,
    on_progress: Box<dyn Fn(f32, f32) -> bool + Sync + Send + 'static>,
) -> ort::Result<VecDeque<f32>> {
    let mut result = VecDeque::new();
    let mut generated = 0;
    let mut history: Option<Vec<f32>> = None;

    println!(
        "Starting infinite generation: total={}s, chunk={}s, overlap={}s",
        total_secs, chunksize, overlap
    );

    while generated < total_secs {
        let seconds_left = total_secs - generated;
        let current_chunk_secs = chunksize.min(seconds_left);

        println!(
            "\nGenerating chunk: {}s ({}s remaining)...",
            current_chunk_secs, seconds_left
        );

        let chunk = self.generate_chunk(prompt, current_chunk_secs, history.as_deref())?;
        let chunk_len = chunk.len();

        println!("Chunk generated: {} samples", chunk_len);

        // Update result, avoid duplicate overlap
        if generated == 0 {
            result.extend(chunk.clone());
        } else {
            let skip = overlap * 50; // assuming 50 samples per second
            println!("Skipping {} samples due to overlap", skip);
            result.extend(chunk.iter().skip(skip));
        }

        // Prepare history for next chunk
        let overlap_len = overlap * 50;
        let history_len = chunk_len.min(overlap_len);
        history = Some(
            chunk
                .iter()
                .rev()
                .take(history_len)
                .cloned()
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect(),
        );

        generated += current_chunk_secs;

        println!("Total generated so far: {}s", generated);

        let stop = on_progress(generated as f32, total_secs as f32);
        if stop {
            println!("Stopping early due to on_progress callback.");
            break;
        }
    }

    println!("\nFinished infinite generation: total length = {} samples", result.len());

    Ok(result)
}
}

impl JobProcessor for MusicGenJobProcessor {
    fn process(
        &self,
        prompt: &str,
        secs: usize,
        on_progress: Box<dyn Fn(f32, f32) -> bool + Sync + Send + 'static>,
    ) -> ort::Result<VecDeque<f32>> {
        self.generate_chunk(prompt, secs, None).map(|mut samples| {
            let _ = on_progress(secs as f32, secs as f32);
            samples
        })
    }

    fn process_infinite(
        &self,
        prompt: &str,
        total_secs: usize,
        chunksize: usize,
        overlap: usize,
        on_progress: Box<dyn Fn(f32, f32) -> bool + Sync + Send + 'static>,
    ) -> ort::Result<VecDeque<f32>> {
        self.process_infinite(prompt, total_secs, chunksize, overlap, on_progress)
    }
}
