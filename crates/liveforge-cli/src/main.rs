use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use liveforge_core::{plan_conversion, ConversionOptions, QualityMode};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "liveforge", version, about = "Quality-first Live Photo tooling")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Inspect source media with ffprobe.
    Analyze {
        input: PathBuf,
    },
    /// Produce a conversion plan without modifying media.
    Plan {
        input: PathBuf,
        #[arg(long)]
        key_time: f64,
        #[arg(long, default_value_t = 3.0)]
        duration: f64,
        #[arg(long, value_enum, default_value_t = QualityArg::Original)]
        quality: QualityArg,
        #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
        preserve_audio: bool,
    },
    /// Perform basic checks on a generated resource pair.
    Verify {
        photo: PathBuf,
        motion: PathBuf,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum QualityArg {
    Original,
    MaximumCompatibility,
}

impl From<QualityArg> for QualityMode {
    fn from(value: QualityArg) -> Self {
        match value {
            QualityArg::Original => Self::Original,
            QualityArg::MaximumCompatibility => Self::MaximumCompatibility,
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Analyze { input } => {
            let media = liveforge_media::probe(&input)
                .with_context(|| format!("could not analyze {}", input.display()))?;
            print_json(&media)?;
        }
        Commands::Plan {
            input,
            key_time,
            duration,
            quality,
            preserve_audio,
        } => {
            let media = liveforge_media::probe(&input)
                .with_context(|| format!("could not analyze {}", input.display()))?;
            let plan = plan_conversion(
                &media,
                &ConversionOptions {
                    key_time_seconds: key_time,
                    duration_seconds: duration,
                    quality: quality.into(),
                    preserve_audio,
                },
            )?;
            print_json(&plan)?;
        }
        Commands::Verify { photo, motion } => {
            let report = liveforge_validator::validate_basic_pair(&photo, &motion);
            print_json(&report)?;
            if !report.passed {
                anyhow::bail!("basic validation failed");
            }
        }
    }
    Ok(())
}

fn print_json(value: &impl serde::Serialize) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}
