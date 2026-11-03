use std::{fs, path::Path};

use anyhow::{Context, Result};
use clap::Parser;

use kopitiam::EngineOptions;

#[derive(Parser, Debug)]
#[command(name = "kopitiam", about = "Run kopitiam engine")]
pub struct CliOptions {
  /// Path to the JavaScript script file
  #[arg(value_name = "SCRIPT")]
  pub script_path: String,

  #[arg(long, value_parser = parse_dimension, requires = "height")]
  pub width: Option<usize>,

  #[arg(long, value_parser = parse_dimension, requires = "width")]
  pub height: Option<usize>,

  #[arg(long, value_name = "WIDTHxHEIGHT", value_parser = parse_size)]
  pub size: Option<(usize, usize)>,
}

impl From<&CliOptions> for EngineOptions {
  fn from(value: &CliOptions) -> Self {
    let mut engine_options = EngineOptions::default();
    engine_options = engine_options.with_console(true);

    if let Some((width, height)) = value.size {
      engine_options = engine_options.with_canvas_size(width, height);
    } else if let (Some(width), Some(height)) = (value.width, value.height) {
      engine_options = engine_options.with_canvas_size(width, height);
    }

    engine_options
  }
}

fn main() -> Result<()> {
  let cli = CliOptions::parse();
  let script_path = &cli.script_path;
  let engine_options = EngineOptions::from(&cli);

  run_script_file_with_options(script_path, engine_options)
}

fn run_script_file_with_options(path: impl AsRef<Path>, options: EngineOptions) -> Result<()> {
  let path = path.as_ref();
  let script = fs::read_to_string(path)
    .with_context(|| format!("failed to read JavaScript file `{}`", path.display()))?;
  kopitiam::run_script(&script, options)
}

fn parse_dimension(value: &str) -> Result<usize, String> {
  let dimension = value
    .parse::<usize>()
    .map_err(|_| "must be a positive integer".to_string())?;
  if dimension == 0 {
    return Err("must be greater than zero".to_string());
  }
  Ok(dimension)
}

fn parse_size(value: &str) -> Result<(usize, usize), String> {
  let Some((width, height)) = value.split_once('x').or_else(|| value.split_once('X')) else {
    return Err("must use WIDTHxHEIGHT, for example 960x540".to_string());
  };
  let width = parse_dimension(width).map_err(|e| format!("width {e}"))?;
  let height = parse_dimension(height).map_err(|e| format!("height {e}"))?;
  Ok((width, height))
}
