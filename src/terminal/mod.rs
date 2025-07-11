use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use regex::Regex;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::fmt::Write;
use std::path::PathBuf;
use std::str::FromStr;

use crate::audio::{AudioManager, AudioStream};
use crate::backend::audio_generation_backend::JobProcessor;

pub struct RunTerminalOptions {
    pub init_prompt: String,
    pub init_secs: usize,
    pub init_output: String,
    pub no_playback: bool,
    pub no_interactive: bool,
    pub infinite: bool,
    pub chunksize: usize,
    pub overlap: usize,
}

pub async fn run_terminal_loop<T: JobProcessor>(
    root: PathBuf,
    processor: T,
    opts: RunTerminalOptions,
) -> anyhow::Result<()> {
    let secs_re = Regex::new("--secs[ =](\\d+)")?;
    let output_re = Regex::new(r"--output[ =]([.a-zA-Z_-]+)")?;

    let audio_player = AudioManager::default();
    #[allow(unused_variables)]
    let mut curr_stream: Option<AudioStream> = None;
    let mut prompt = opts.init_prompt;
    let mut secs = opts.init_secs;
    let mut output = opts.init_output;

    let mut rl = DefaultEditor::new()?;
    let _ = rl.load_history(&root.join("history.txt"));
    let _ = rl.add_history_entry(&prompt);

    if opts.infinite {
    if opts.init_secs <= 30 {
        println!(
            "[WARNING] --infinite flag ignored: total duration ({}s) is too short.",
            opts.init_secs
        );
    }

    if opts.chunksize > 30 {
        return Err(anyhow::anyhow!(
            "Chunk size ({}) must be <= 30 seconds.",
            opts.chunksize
        ));
    }

    if opts.overlap >= opts.chunksize {
        return Err(anyhow::anyhow!(
            "Overlap ({}) must be less than chunk size ({}).",
            opts.overlap, opts.chunksize
        ));
    }
}

    loop {
        if prompt.is_empty() {
            prompt = match rl.readline(">>> ") {
                Ok(line) => line,
                Err(ReadlineError::Interrupted) => return Ok(()),
                Err(ReadlineError::Eof) => return Ok(()),
                Err(err) => return Err(anyhow::anyhow!(err)),
            };
            secs = capture(&secs_re, &prompt).unwrap_or(secs);
            output = capture(&output_re, &prompt).unwrap_or(output);
        }

        if prompt.is_empty() {
            continue;
        }

        let _ = rl.add_history_entry(&prompt);

        if prompt == "exit" {
            return Ok(());
        }

        let bar = fixed_bar("Generating audio", 1);

        let samples = if opts.infinite {
            processor.process_infinite(
                &prompt,
                secs,
                opts.chunksize,
                opts.overlap,
                Box::new(move |elapsed, total| {
                    bar.set_length(total as u64);
                    bar.set_position(elapsed as u64);
                    false
                }),
            )?
        } else {
            processor.process(
                &prompt,
                secs,
                Box::new(move |elapsed, total| {
                    bar.set_length(total as u64);
                    bar.set_position(elapsed as u64);
                    false
                }),
            )?
        };

        if !opts.no_playback {
            let samples_copy = samples.clone();
            let stream = audio_player.play_from_queue(samples_copy);
            #[allow(unused_assignments)]
            if let Ok(stream) = stream {
                curr_stream = Some(stream);
            }
        }

        if !output.ends_with(".wav") {
            output += ".wav";
        }

        let bytes = audio_player.to_wav(samples.clone())?;
        tokio::fs::write(&output, bytes).await?;

        prompt = "".into();
        if opts.no_interactive {
            break;
        }
    }

    Ok(())
}

pub fn fixed_bar(prefix: impl Into<String>, len: usize) -> ProgressBar {
    let pb = ProgressBar::new(len as u64);
    pb.set_style(
        ProgressStyle::with_template(
            &(prefix.into()
                + " {spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] ({eta})"),
        )
        .unwrap()
        .with_key("eta", |state: &ProgressState, w: &mut dyn Write| {
            write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap()
        })
        .progress_chars("#>-"),
    );
    pb
}

fn capture<T: FromStr>(re: &Regex, text: &str) -> Option<T> {
    if let Some(Some(capture)) = re.captures(text).map(|c| c.get(1)) {
        if let Ok(v) = T::from_str(capture.as_str()) {
            return Some(v);
        }
    }
    None
}
