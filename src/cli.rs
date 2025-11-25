use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::client::NhkRadioClient;
use crate::player::Player;

#[derive(Parser)]
#[command(name = "nhk-radio-player")]
#[command(about = "A CLI radio player for NHK Radio", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Play radio stream
    Play {
        /// Area code (e.g., "130" for Tokyo)
        area: String,
        /// Channel type: r1, r2, or fm
        kind: String,
    },
    /// List available areas
    Area,
    /// Show program information for an area
    Program {
        /// Area code
        area: String,
    },
    /// List all available streams
    List,
}

pub async fn run_cli() -> Result<()> {
    env_logger::init();

    let cli = Cli::parse();
    let client = NhkRadioClient::new();

    match cli.command {
        Commands::Play { area, kind } => {
            let config = client.fetch_config().await?;

            for data in &config.stream_url.data {
                if data.area == area {
                    let m3u8_url = match kind.as_str() {
                        "r1" => &data.r1hls,
                        "r2" => &data.r2hls,
                        "fm" => &data.fmhls,
                        _ => anyhow::bail!("Invalid kind: {}. Must be one of: r1, r2, fm", kind),
                    };

                    println!("Playing {} Radio ({}) from area {}", kind.to_uppercase(), data.areajp, area);
                    println!("Stream URL: {}", m3u8_url);
                    println!("Press Ctrl+C to stop playback.");

                    let player = Player::new();
                    return player.play_live(m3u8_url).await;
                }
            }

            anyhow::bail!("Area not found: {}", area);
        }

        Commands::Area => {
            let config = client.fetch_config().await?;
            println!("Available areas:");
            println!("{:<10} {}", "Area Code", "Area Name");
            println!("{:-<40}", "");
            for data in &config.stream_url.data {
                println!("{:<10} {}", data.area, data.areajp);
            }
            Ok(())
        }

        Commands::Program { area } => {
            let config = client.fetch_config().await?;

            for data in &config.stream_url.data {
                if data.area == area {
                    let area_key = &data.areakey;
                    let url = config
                        .url_program_noa
                        .replace("//", "https://")
                        .replace("{area}", area_key);

                    let program = client.fetch_program(&url).await?;

                    println!("\n=== R1 Current Program ===");
                    if let Some(ref present) = program.r1.present {
                        if let Some(ref about) = present.about {
                            println!("ID: {}", about.id);
                            println!("Name: {}", about.name);
                            println!("Description: {}", about.description);
                        } else {
                            println!("No program information available");
                        }
                    } else {
                        println!("No current program");
                    }

                    println!("\n=== R2 Current Program ===");
                    if let Some(ref present) = program.r2.present {
                        if let Some(ref about) = present.about {
                            println!("ID: {}", about.id);
                            println!("Name: {}", about.name);
                            println!("Description: {}", about.description);
                        } else {
                            println!("No program information available");
                        }
                    } else {
                        println!("No current program");
                    }

                    println!("\n=== FM Current Program ===");
                    if let Some(ref present) = program.r3.present {
                        if let Some(ref about) = present.about {
                            println!("ID: {}", about.id);
                            println!("Name: {}", about.name);
                            println!("Description: {}", about.description);
                        } else {
                            println!("No program information available");
                        }
                    } else {
                        println!("No current program");
                    }

                    return Ok(());
                }
            }

            anyhow::bail!("Area not found: {}", area);
        }

        Commands::List => {
            let config = client.fetch_config().await?;
            println!("Available streams:");
            println!();
            for data in &config.stream_url.data {
                println!("Area: {} ({})", data.area, data.areajp);
                println!("  R1 HLS: {}", data.r1hls);
                println!("  R2 HLS: {}", data.r2hls);
                println!("  FM HLS: {}", data.fmhls);
                println!();
            }
            Ok(())
        }
    }
}
