use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowConfig {
  pub title: String,
  pub width: usize,
  pub height: usize,
  #[serde(default)]
  pub resizable: bool,
  #[serde(default)]
  pub fullscreen: bool,
  #[serde(default)]
  pub scale_mode: Option<String>,
}

impl Default for WindowConfig {
  fn default() -> Self {
    Self {
      title: "Canvas".to_owned(),
      width: 800,
      height: 450,
      resizable: false,
      fullscreen: false,
      scale_mode: Some("showAll".to_owned()),
    }
  }
}

impl WindowConfig {
  pub fn normalized(mut self) -> Self {
    self.width = self.width.clamp(64, 7680);
    self.height = self.height.clamp(64, 4320);
    self
  }
}
