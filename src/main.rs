use baad::utils::json;
use baad::catalog::{CatalogParser};
use baad::download::{ResourceDownloader, ResourceCategory};
use baad::helpers::{ServerConfig, ServerRegion, Platform, BuildType};
use baad::download::ResourceFilter;

use eyre::Result;
use std::path::PathBuf;

use baad_core::config::{init_logging, LoggingConfig};
use baad_core::info;

use clap::Parser;

pub struct PNGFilter {
	enabled: bool
}

impl ResourceFilter for PNGFilter {
	fn matches(&self, path: &str) -> bool {
		path.ends_with(".bundle")
		&& (
			!self.enabled
			||
			path.contains("textures")
			&& (path.contains("uis") 
				|| path.contains("mx-addressableasset-ui")
			)
			&& !(path.contains("mx-spine")
				|| path.contains("mx-npcs")
				|| path.contains("mx-obstacles")
				|| path.contains("mx-cafe")
				|| path.contains("mx-characters")
			)
		)
	}
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    catalog_root_url: String,
    #[arg(short, long)]
    filter: bool
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let config = LoggingConfig {
        verbose_mode: false,
        enable_debug: false,
        ..LoggingConfig::default()
    };
    init_logging(config)?;
	
	let args = Args::parse();
	info!("Using catalog url {}", args.catalog_root_url);
	let addressable_catalog_root = args.catalog_root_url;
	
	json::update_api_data(|data| {
		data.japan.catalog_url = addressable_catalog_root.to_string();
	})
	.await?;

    let config_android = ServerConfig::new(ServerRegion::Japan, Some(Platform::Android), Some(BuildType::Standard))?;
    let catalog_parser_android = CatalogParser::new(config_android.clone())?;
    catalog_parser_android.process_catalogs().await?;

    let downloader = ResourceDownloader::new(
        Some(PathBuf::from("./Android")), 
        config_android.clone()
    ).await?;
	
	info!("Downloading Android Assets");
	downloader.download::<PNGFilter>(ResourceCategory::Assets, Some(PNGFilter {enabled: args.filter})).await?;
	
	let config_ios = ServerConfig::new(ServerRegion::Japan, Some(Platform::Ios), Some(BuildType::Standard))?;
    let catalog_parser_ios = CatalogParser::new(config_ios.clone())?;
    catalog_parser_ios.process_catalogs().await?;

	info!("Downloading iOS Assets");
    let downloader = ResourceDownloader::new(
        Some(PathBuf::from("./iOS")), 
        config_android.clone()
    ).await?;
	
	downloader.download::<PNGFilter>(ResourceCategory::Assets, Some(PNGFilter {enabled: args.filter})).await?;

    Ok(())
}