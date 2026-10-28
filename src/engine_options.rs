use crate::{
  native_function_binding::{NativeFunction, NativeFunctionBinding},
  window_config::WindowConfig,
};

#[derive(Debug, Clone)]
pub struct EngineOptions {
  pub console: bool,
  pub window: WindowConfig,
  pub native_functions: Vec<NativeFunctionBinding>,
}

impl Default for EngineOptions {
  fn default() -> Self {
    Self {
      console: cfg!(debug_assertions),
      window: WindowConfig::default(),
      native_functions: Vec::new(),
    }
  }
}

impl EngineOptions {
  pub fn with_title(mut self, title: impl Into<String>) -> Self {
    self.window.title = title.into();
    self
  }

  pub fn with_canvas_size(mut self, width: usize, height: usize) -> Self {
    self.window.width = width;
    self.window.height = height;
    self
  }

  pub fn with_resizable(mut self, resizable: bool) -> Self {
    self.window.resizable = resizable;
    self
  }

  pub fn with_fullscreen(mut self, fullscreen: bool) -> Self {
    self.window.fullscreen = fullscreen;
    self
  }

  pub fn with_scale_mode(mut self, scale_mode: impl Into<String>) -> Self {
    self.window.scale_mode = Some(scale_mode.into());
    self
  }

  pub fn with_window(mut self, window: WindowConfig) -> Self {
    self.window = window;
    self
  }

  pub fn with_native_function(mut self, name: impl Into<String>, function: NativeFunction) -> Self {
    self.add_native_function(name, function);
    self
  }

  fn add_native_function(&mut self, name: impl Into<String>, function: NativeFunction) {
    self
      .native_functions
      .push(NativeFunctionBinding::new(name, function));
  }

  pub fn with_native_function_arity(
    mut self,
    name: impl Into<String>,
    arity: usize,
    function: NativeFunction,
  ) -> Self {
    self.add_native_function_arity(name, arity, function);
    self
  }

  fn add_native_function_arity(
    &mut self,
    name: impl Into<String>,
    arity: usize,
    function: NativeFunction,
  ) {
    self
      .native_functions
      .push(NativeFunctionBinding::with_arity(name, arity, function));
  }
}
