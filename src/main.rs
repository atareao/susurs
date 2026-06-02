use bzip2::read::BzDecoder;
use clap::Parser;
use dirs::data_dir;
use sherpa_onnx::{OfflineTts, OfflineTtsConfig, OfflineTtsModelConfig, OfflineTtsVitsModelConfig};
use shine_rs::{Mp3Encoder, Mp3EncoderConfig, StereoMode};
use std::fs::File;
use std::io::{self, Read, Write};
use tar::Archive;

// Model configuration
struct TtsModel {
    name: &'static str,
    model_file: &'static str,
    tokens_file: &'static str,
    archive_url: &'static str,
}

const MODELS: &[TtsModel] = &[
    TtsModel {
        name: "vits-piper-es_AR-daniela-high",
        model_file: "es_AR-daniela-high.onnx",
        tokens_file: "tokens.txt",
        archive_url: "https://github.com/k2-fsa/sherpa-onnx/releases/download/tts-models/vits-piper-es_AR-daniela-high.tar.bz2",
    },
    TtsModel {
        name: "vits-piper-es_ES-carlfm-x_low",
        model_file: "es_ES-carlfm-x_low.onnx",
        tokens_file: "tokens.txt",
        archive_url: "https://github.com/k2-fsa/sherpa-onnx/releases/download/tts-models/vits-piper-es_ES-carlfm-x_low.tar.bz2",
    },
    TtsModel {
        name: "vits-piper-es_ES-davefx-medium",
        model_file: "es_ES-davefx-medium.onnx",
        tokens_file: "tokens.txt",
        archive_url: "https://github.com/k2-fsa/sherpa-onnx/releases/download/tts-models/vits-piper-es_ES-davefx-medium.tar.bz2",
    },
    TtsModel {
        name: "vits-piper-es_ES-glados-medium",
        model_file: "es_ES-glados-medium.onnx",
        tokens_file: "tokens.txt",
        archive_url: "https://github.com/k2-fsa/sherpa-onnx/releases/download/tts-models/vits-piper-es_ES-glados-medium.tar.bz2",
    },
    TtsModel {
        name: "vits-piper-es_ES-miro-high",
        model_file: "es_ES-miro-high.onnx",
        tokens_file: "tokens.txt",
        archive_url: "https://github.com/k2-fsa/sherpa-onnx/releases/download/tts-models/vits-piper-es_ES-miro-high.tar.bz2",
    },
    TtsModel {
        name: "vits-piper-es_ES-sharvard-medium",
        model_file: "es_ES-sharvard-medium.onnx",
        tokens_file: "tokens.txt",
        archive_url: "https://github.com/k2-fsa/sherpa-onnx/releases/download/tts-models/vits-piper-es_ES-sharvard-medium.tar.bz2",
    },
    TtsModel {
        name: "vits-piper-es_MX-ald-medium",
        model_file: "es_MX-ald-medium.onnx",
        tokens_file: "tokens.txt",
        archive_url: "https://github.com/k2-fsa/sherpa-onnx/releases/download/tts-models/vits-piper-es_MX-ald-medium.tar.bz2",
    },
    TtsModel {
        name: "vits-piper-es_MX-claude-high",
        model_file: "es_MX-claude-high.onnx",
        tokens_file: "tokens.txt",
        archive_url: "https://github.com/k2-fsa/sherpa-onnx/releases/download/tts-models/vits-piper-es_MX-claude-high.tar.bz2",
    },
    TtsModel {
        name: "supertonic-3-es",
        model_file: "supertonic-3-es.onnx",
        tokens_file: "tokens.txt",
        archive_url: "https://github.com/k2-fsa/sherpa-onnx/releases/download/tts-models/supertonic-3-es.tar.bz2",
    },
];

// Get the XDG data directory for model caching
fn get_model_cache_dir() -> std::path::PathBuf {
    let mut path = data_dir().unwrap_or(std::path::PathBuf::from("~/.local/share"));
    path.push("tts-models");
    path
}

// Download and extract model if not present
fn ensure_model(model_name: &str) -> Result<(String, String, String), Box<dyn std::error::Error>> {
    let cache_dir = get_model_cache_dir();
    let model_dir = cache_dir.join(model_name);

    // Check if model files exist
    let model_config = MODELS
        .iter()
        .find(|m| m.name == model_name)
        .ok_or_else(|| format!("Model {} not found", model_name))?;

    let model_path = model_dir.join(model_config.model_file);
    let tokens_path = model_dir.join(model_config.tokens_file);

    if model_path.exists() && tokens_path.exists() {
        return Ok((
            model_path.to_string_lossy().to_string(),
            tokens_path.to_string_lossy().to_string(),
            model_dir.to_string_lossy().to_string(),
        ));
    }

    // Create cache directory if it doesn't exist
    std::fs::create_dir_all(&cache_dir)?;

    println!("Downloading model {}...", model_name);

    // Download the archive
    let archive_data = ureq::get(model_config.archive_url)
        .timeout(std::time::Duration::from_secs(60))
        .call()?
        .into_reader();

    // Extract the archive
    let decoder = BzDecoder::new(archive_data);
    let mut archive = Archive::new(decoder);
    archive.unpack(&cache_dir)?;

    // Verify extraction
    if !model_path.exists() || !tokens_path.exists() {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Model files not found after extraction",
        )));
    }

    Ok((
        model_path.to_string_lossy().to_string(),
        tokens_path.to_string_lossy().to_string(),
        model_dir.to_string_lossy().to_string(),
    ))
}

// Model management functions
fn delete_model(model_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let cache_dir = get_model_cache_dir();
    let model_dir = cache_dir.join(model_name);

    if !model_dir.exists() {
        eprintln!("Error: Model '{}' is not cached", model_name);
        return Ok(());
    }

    println!("Deleting model: {}", model_name);
    std::fs::remove_dir_all(&model_dir)?;
    println!("Success! Model '{}' deleted", model_name);
    Ok(())
}

fn show_model_info(model_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let cache_dir = get_model_cache_dir();
    let model_dir = cache_dir.join(model_name);

    // Find model in MODELS list
    let model_config = MODELS
        .iter()
        .find(|m| m.name == model_name)
        .ok_or_else(|| format!("Model '{}' not found in available models", model_name))?;

    println!("Model Information:");
    println!("  Name: {}", model_config.name);
    println!("  Model file: {}", model_config.model_file);
    println!("  Tokens file: {}", model_config.tokens_file);
    println!("  Download URL: {}", model_config.archive_url);

    // Check if model is cached
    if model_dir.exists() {
        let model_path = model_dir.join(model_config.model_file);
        let tokens_path = model_dir.join(model_config.tokens_file);

        println!("\n  Status: Cached");
        println!("  Cache directory: {}", model_dir.display());

        if model_path.exists() {
            let size = std::fs::metadata(&model_path)?.len();
            println!("  Model size: {:.2} MB", size as f64 / 1_048_576.0);
        }

        if tokens_path.exists() {
            let size = std::fs::metadata(&tokens_path)?.len();
            println!("  Tokens size: {:.2} KB", size as f64 / 1024.0);
        }
    } else {
        println!("\n  Status: Not cached (will be downloaded on first use)");
    }

    Ok(())
}

fn update_models() -> Result<(), Box<dyn std::error::Error>> {
    println!("Checking for model updates...");
    println!("Note: This will re-download all cached models to ensure latest version.");

    let cache_dir = get_model_cache_dir();
    let mut updated = 0;
    let mut failed = 0;

    for model in MODELS {
        let model_dir = cache_dir.join(model.name);

        if !model_dir.exists() {
            continue; // Skip if not cached
        }

        println!("\nUpdating model: {}", model.name);

        // Remove old version
        std::fs::remove_dir_all(&model_dir).ok();

        // Re-download
        match ensure_model(model.name) {
            Ok(_) => {
                println!("  ✓ Updated successfully");
                updated += 1;
            }
            Err(e) => {
                eprintln!("  ✗ Failed: {}", e);
                failed += 1;
            }
        }
    }

    println!("\nUpdate complete: {} updated, {} failed", updated, failed);
    Ok(())
}

fn clean_cache() -> Result<(), Box<dyn std::error::Error>> {
    let cache_dir = get_model_cache_dir();

    if !cache_dir.exists() {
        println!("Cache directory does not exist: {}", cache_dir.display());
        return Ok(());
    }

    println!("Cleaning cache directory: {}", cache_dir.display());

    // List models to be deleted
    let mut count = 0;
    for entry in std::fs::read_dir(&cache_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            println!(
                "  Deleting: {}",
                path.file_name().unwrap().to_string_lossy()
            );
            count += 1;
        }
    }

    // Delete all subdirectories (models)
    std::fs::remove_dir_all(&cache_dir)?;
    std::fs::create_dir_all(&cache_dir)?;

    println!("Success! Deleted {} cached models", count);
    Ok(())
}

#[derive(Parser, Debug)]
#[command(
    version,
    about = "Convert text to MP3 audio using local AI TTS models",
    long_about = "A fully offline Text-to-Speech CLI tool that converts text to MP3 audio.\n\
                   Supports multiple Spanish voices and customizable speech parameters.\n\
                   Models are automatically downloaded and cached on first use."
)]
struct Args {
    /// Text to convert to speech (mutually exclusive with --input)
    #[arg(short, long, group = "input_source", value_name = "TEXT")]
    text: Option<String>,

    /// Read text from file (mutually exclusive with --text)
    #[arg(short = 'i', long, group = "input_source", value_name = "FILE")]
    input: Option<String>,

    /// Output MP3 file path (default: output.mp3)
    #[arg(short, long, default_value = "output.mp3", value_name = "FILE")]
    output: String,

    /// Write MP3 to stdout instead of file
    #[arg(long, conflicts_with = "output")]
    stdout: bool,

    /// TTS model to use for speech synthesis
    #[arg(
        long,
        default_value = "vits-piper-es_AR-daniela-high",
        value_name = "MODEL"
    )]
    model: String,

    /// Speaking rate/speed (0.5=slow, 1.0=normal, 2.0=fast)
    #[arg(long, value_name = "SPEED")]
    rate: Option<f32>,

    /// Voice pitch/speech characteristics (0.2=low, 0.667=normal, 1.5=high)
    #[arg(long, value_name = "PITCH")]
    pitch: Option<f32>,

    /// Volume scaling factor (0.5=quiet, 1.0=normal, 2.0=loud)
    #[arg(long, value_name = "VOL")]
    volume: Option<f32>,

    /// Noise scale W parameter for prosody variation (default: 0.8)
    #[arg(long, value_name = "SCALE")]
    noise_scale_w: Option<f32>,

    /// Length scale for independent tempo control (default: 1.0)
    #[arg(long, value_name = "SCALE")]
    length_scale: Option<f32>,

    /// Silence scale for pause duration (default: 0.2)
    #[arg(long, value_name = "SCALE")]
    silence_scale: Option<f32>,

    /// Speaker ID for multi-speaker models (e.g., 0, 1, 2)
    #[arg(long, value_name = "ID")]
    sid: Option<i32>,

    /// Add silence at start/end in seconds (e.g., 0.5)
    #[arg(long, default_value = "0", value_name = "SECONDS")]
    silence: f32,

    /// Apply preset configuration (human, fast, slow, quiet, loud)
    #[arg(long, value_name = "PRESET")]
    preset: Option<String>,

    /// Output audio format (mp3, wav, ogg, auto)
    #[arg(long, default_value = "auto", value_name = "FORMAT")]
    format: String,

    /// MP3 bitrate in kbps (128, 192, 320)
    #[arg(long, default_value = "128", value_name = "KBPS")]
    bitrate: u32,

    /// Override output sample rate (e.g., 22050, 44100)
    #[arg(long, value_name = "HZ")]
    sample_rate: Option<u32>,

    /// Normalize audio volume automatically
    #[arg(long)]
    normalize: bool,

    /// List all available TTS models
    #[arg(long)]
    list_models: bool,

    /// Delete a cached model
    #[arg(long, value_name = "MODEL")]
    delete_model: Option<String>,

    /// Show information about a specific model
    #[arg(long, value_name = "MODEL")]
    model_info: Option<String>,

    /// Update all cached models to latest version
    #[arg(long)]
    update_models: bool,

    /// Clean all cached models
    #[arg(long)]
    clean_cache: bool,

    /// Dry run: show what would be done without generating audio
    #[arg(long)]
    dry_run: bool,

    /// Estimate audio duration before generation
    #[arg(long)]
    estimate: bool,

    /// Output format for scripting (text, json)
    #[arg(long, default_value = "text", value_name = "FMT")]
    output_format: String,
}

fn read_text(args: &Args) -> Result<String, Box<dyn std::error::Error>> {
    if let Some(text) = &args.text {
        return Ok(text.clone());
    }

    if let Some(input_file) = &args.input {
        let mut file = File::open(input_file)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        return Ok(content);
    }

    // Read from stdin
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;
    Ok(buffer)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();

    // Handle model management commands
    if let Some(model_name) = &args.delete_model {
        return delete_model(model_name);
    }
    if let Some(model_name) = &args.model_info {
        return show_model_info(model_name);
    }
    if args.update_models {
        return update_models();
    }
    if args.clean_cache {
        return clean_cache();
    }

    // Handle --list-models
    if args.list_models {
        println!("Available TTS Models:\n");
        println!("Default: vits-piper-es_AR-daniela-high (Argentine Spanish, high quality)\n");
        for model in MODELS {
            let description = match model.name {
                "vits-piper-es_AR-daniela-high" => {
                    "Argentine Spanish, female, high quality (DEFAULT)"
                }
                "vits-piper-es_ES-carlfm-x_low" => "Spanish (Spain), male, low quality",
                "vits-piper-es_ES-davefx-medium" => "Spanish (Spain), male, medium quality",
                "vits-piper-es_ES-glados-medium" => "Spanish (Spain), female, medium quality",
                "vits-piper-es_ES-miro-high" => "Spanish (Spain), male, high quality",
                "vits-piper-es_ES-sharvard-medium" => "Spanish (Spain), male, medium quality",
                "vits-piper-es_MX-ald-medium" => "Mexican Spanish, male, medium quality",
                "vits-piper-es_MX-claude-high" => "Mexican Spanish, male, high quality",
                "supertonic-3-es" => "Spanish multi-speaker, high quality",
                _ => "Custom model",
            };
            let marker = if model.name == "vits-piper-es_AR-daniela-high" {
                " *"
            } else {
                ""
            };
            println!("  {}{}", model.name, marker);
            println!("      {}\n", description);
        }
        return Ok(());
    }

    // Read text
    let text = read_text(&args)?;
    if text.trim().is_empty() {
        eprintln!("Error: No text provided");
        return Ok(());
    }

    // Apply preset or defaults, then override with explicit args
    let (
        mut rate,
        mut pitch,
        mut volume,
        mut noise_scale_w,
        mut length_scale,
        mut silence_scale,
        mut sid,
    ) = if let Some(preset_name) = &args.preset {
        let (r, p, v, nw, ls, ss, s) = apply_preset(preset_name)?;
        (r, p, v, nw, ls, ss, s)
    } else {
        (1.0, 0.667, 1.0, 0.8, 1.0, 0.2, None)
    };
    if let Some(r) = args.rate {
        rate = r;
    }
    if let Some(p) = args.pitch {
        pitch = p;
    }
    if let Some(v) = args.volume {
        volume = v;
    }
    if let Some(nw) = args.noise_scale_w {
        noise_scale_w = nw;
    }
    if let Some(ls) = args.length_scale {
        length_scale = ls;
    }
    if let Some(ss) = args.silence_scale {
        silence_scale = ss;
    }
    if let Some(s) = args.sid {
        sid = Some(s);
    }

    // Dry run
    if args.dry_run {
        println!("\n=== DRY RUN ===");
        println!("Would generate audio with:");
        println!("  Model: {}", args.model);
        println!("  Text length: {} characters", text.len());
        println!("  Rate: {}", rate);
        println!("  Pitch: {}", pitch);
        println!("  Volume: {}", volume);
        println!("\nNo audio generated (dry-run mode)");
        return Ok(());
    }

    // Estimate
    if args.estimate {
        let chars = text.chars().count();
        let estimated_seconds = (chars as f32 / 15.0) / rate;
        if args.output_format == "json" {
            let estimate = serde_json::json!({"estimate": {"chars": chars, "estimated_seconds": estimated_seconds}});
            println!("{}", serde_json::to_string_pretty(&estimate)?);
        } else {
            println!("\n=== ESTIMATE ===");
            println!("Estimated duration: {:.1} seconds", estimated_seconds);
        }
        return Ok(());
    }

    // Ensure model
    let (model_path, tokens_path, model_dir) = ensure_model(&args.model)?;

    // Configure TTS
    let vits_config = OfflineTtsVitsModelConfig {
        model: Some(model_path),
        lexicon: None,
        tokens: Some(tokens_path),
        data_dir: Some(format!("{}/espeak-ng-data", model_dir)),
        noise_scale: pitch,
        noise_scale_w,
        length_scale: 1.0 / rate * length_scale,
        dict_dir: None,
    };
    let config = OfflineTtsConfig {
        model: OfflineTtsModelConfig {
            vits: vits_config,
            ..Default::default()
        },
        rule_fsts: None,
        max_num_sentences: 1,
        rule_fars: None,
        silence_scale,
    };

    println!("Initializing local engine...");
    let tts = OfflineTts::create(&config).expect("Failed to create TTS engine");

    println!("Generating audio...");
    let gen_config = sherpa_onnx::GenerationConfig {
        speed: rate,
        sid: sid.unwrap_or(0),
        ..Default::default()
    };
    let callback: Option<fn(&[f32], f32) -> bool> = None;
    let audio = tts
        .generate_with_config(&text, &gen_config, callback)
        .expect("Failed to generate audio");

    // Process samples
    let mut samples: Vec<f32> = audio
        .samples()
        .iter()
        .map(|&x| (x * volume).clamp(-1.0, 1.0))
        .collect();
    let sample_rate = audio.sample_rate() as u32;

    if args.normalize {
        tracing::debug!("Applying normalization...");
        samples = normalize_audio(&samples);
    }
    if args.silence > 0.0 {
        let silence_samples = (sample_rate as f32 * args.silence) as usize;
        let mut result = vec![0.0; silence_samples];
        result.extend_from_slice(&samples);
        result.extend(vec![0.0; silence_samples]);
        samples = result;
    }
    let output_sample_rate = args.sample_rate.unwrap_or(sample_rate);
    let samples = if output_sample_rate != sample_rate {
        tracing::debug!(
            "Resampling from {} to {} Hz...",
            sample_rate,
            output_sample_rate
        );
        resample(&samples, sample_rate, output_sample_rate)?
    } else {
        samples
    };

    // Determine output format
    let output_format = if args.format == "auto" {
        if args.stdout {
            "mp3".to_string()
        } else {
            match std::path::Path::new(&args.output)
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("mp3")
                .to_lowercase()
                .as_str()
            {
                "wav" => "wav".to_string(),
                _ => "mp3".to_string(),
            }
        }
    } else {
        args.format.clone()
    };

    tracing::debug!("Output format: {}", output_format);
    match output_format.as_str() {
        "wav" => write_wav(&samples, output_sample_rate, &args)?,
        _ => write_mp3(&samples, output_sample_rate, &args)?,
    }

    Ok(())
}

// --- Preset configurations ---
type PresetResult = Result<(f32, f32, f32, f32, f32, f32, Option<i32>), Box<dyn std::error::Error>>;

fn apply_preset(
    preset_name: &str,
) -> PresetResult {
    match preset_name.to_lowercase().as_str() {
        "human" => {
            println!("Applying 'human' preset for natural voice...");
            Ok((0.95, 0.5, 1.0, 0.9, 1.0, 0.3, None))
        }
        "fast" => {
            println!("Applying 'fast' preset...");
            Ok((1.5, 0.667, 1.0, 0.8, 1.0, 0.2, None))
        }
        "slow" => {
            println!("Applying 'slow' preset...");
            Ok((0.7, 0.667, 1.0, 0.8, 1.0, 0.2, None))
        }
        "quiet" => {
            println!("Applying 'quiet' preset...");
            Ok((1.0, 0.667, 0.5, 0.8, 1.0, 0.2, None))
        }
        "loud" => {
            println!("Applying 'loud' preset...");
            Ok((1.0, 0.667, 2.0, 0.8, 1.0, 0.2, None))
        }
        _ => Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!(
                "Unknown preset: '{}'. Available: human, fast, slow, quiet, loud",
                preset_name
            ),
        ))),
    }
}

// Audio processing helper functions
fn normalize_audio(samples: &[f32]) -> Vec<f32> {
    if samples.is_empty() {
        return samples.to_vec();
    }
    let max_amplitude = samples
        .iter()
        .map(|&x| x.abs())
        .fold(0.0f32, |a, b| a.max(b));
    if max_amplitude < 0.001 {
        return samples.to_vec();
    }
    let gain = 0.95 / max_amplitude;
    samples.iter().map(|&x| x * gain).collect()
}

fn resample(
    samples: &[f32],
    input_rate: u32,
    output_rate: u32,
) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
    if input_rate == output_rate {
        return Ok(samples.to_vec());
    }
    let ratio = output_rate as f32 / input_rate as f32;
    let output_len = (samples.len() as f32 * ratio) as usize;
    let mut result = Vec::with_capacity(output_len);
    for i in 0..output_len {
        let src_index = i as f32 / ratio;
        let index_floor = src_index.floor() as usize;
        let index_ceil = (index_floor + 1).min(samples.len() - 1);
        let frac = src_index - index_floor as f32;
        if index_floor < samples.len() - 1 {
            result.push(samples[index_floor] * (1.0 - frac) + samples[index_ceil] * frac);
        } else {
            result.push(samples[index_floor]);
        }
    }
    Ok(result)
}

fn write_wav(
    samples: &[f32],
    sample_rate: u32,
    args: &Args,
) -> Result<(), Box<dyn std::error::Error>> {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    if args.stdout {
        let mut cursor = std::io::Cursor::new(Vec::new());
        let mut writer = hound::WavWriter::new(&mut cursor, spec)?;
        for &sample in samples {
            writer.write_sample((sample.clamp(-1.0, 1.0) * 32767.0) as i16)?;
        }
        writer.finalize()?;
        io::stdout().write_all(&cursor.into_inner())?;
        println!("Success! Audio written to stdout");
    } else {
        let mut writer = hound::WavWriter::create(&args.output, spec)?;
        for &sample in samples {
            writer.write_sample((sample.clamp(-1.0, 1.0) * 32767.0) as i16)?;
        }
        writer.finalize()?;
        println!("Success! Saved speech to '{}'", args.output);
    }
    Ok(())
}

fn write_mp3(
    samples: &[f32],
    sample_rate: u32,
    args: &Args,
) -> Result<(), Box<dyn std::error::Error>> {
    let pcm: Vec<i16> = samples
        .iter()
        .map(|&x| (x.clamp(-1.0, 1.0) * 32767.0) as i16)
        .collect();
    let bitrate = match args.bitrate {
        64 | 96 | 128 | 192 | 256 | 320 => args.bitrate,
        _ => {
            eprintln!(
                "Warning: Unsupported bitrate {}, using 128 kbps",
                args.bitrate
            );
            128
        }
    };
    let config = Mp3EncoderConfig::new()
        .sample_rate(sample_rate)
        .bitrate(bitrate)
        .channels(1)
        .stereo_mode(StereoMode::Mono);
    let mut encoder = Mp3Encoder::new(config)?;
    let mut mp3 = Vec::new();
    for frame in encoder.encode_interleaved(&pcm)? {
        mp3.extend_from_slice(&frame);
    }
    mp3.extend_from_slice(&encoder.finish()?);
    if args.stdout {
        io::stdout().write_all(&mp3)?;
        println!("Success! Audio written to stdout");
    } else {
        File::create(&args.output)?.write_all(&mp3)?;
        println!("Success! Saved speech to '{}'", args.output);
    }
    Ok(())
}
