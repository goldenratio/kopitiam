use std::{
  env,
  path::{Path as FsPath, PathBuf},
};

use anyhow::{Result, anyhow, bail};

pub fn resolve_asset_path(src: &str) -> PathBuf {
  let path = FsPath::new(src);
  if path.is_absolute() {
    path.to_path_buf()
  } else {
    env::current_dir()
      .unwrap_or_else(|_| PathBuf::from("."))
      .join(path)
  }
}

pub fn decode_data_url(src: &str) -> Result<Option<Vec<u8>>> {
  let Some(rest) = src.strip_prefix("data:") else {
    return Ok(None);
  };
  let (metadata, payload) = rest
    .split_once(',')
    .ok_or_else(|| anyhow!("data URL is missing a comma separator"))?;
  if !metadata
    .split(';')
    .any(|part| part.eq_ignore_ascii_case("base64"))
  {
    bail!("only base64 data URLs are supported");
  }

  decode_base64(payload).map(Some)
}

fn decode_base64(input: &str) -> Result<Vec<u8>> {
  let mut output = Vec::with_capacity(input.len() * 3 / 4);
  let mut quartet = [0_u8; 4];
  let mut count = 0;

  for byte in input.bytes().filter(|byte| !byte.is_ascii_whitespace()) {
    quartet[count] = match byte {
      b'A'..=b'Z' => byte - b'A',
      b'a'..=b'z' => byte - b'a' + 26,
      b'0'..=b'9' => byte - b'0' + 52,
      b'+' | b'-' => 62,
      b'/' | b'_' => 63,
      b'=' => 64,
      _ => bail!("invalid base64 character in data URL"),
    };
    count += 1;

    if count == 4 {
      if quartet[0] == 64 || quartet[1] == 64 || (quartet[2] == 64 && quartet[3] != 64) {
        bail!("invalid base64 padding in data URL");
      }

      output.push((quartet[0] << 2) | (quartet[1] >> 4));
      if quartet[2] != 64 {
        output.push((quartet[1] << 4) | (quartet[2] >> 2));
      }
      if quartet[3] != 64 {
        output.push((quartet[2] << 6) | quartet[3]);
      }
      count = 0;
    }
  }

  if count != 0 {
    bail!("base64 data URL length is not a multiple of 4");
  }

  Ok(output)
}
