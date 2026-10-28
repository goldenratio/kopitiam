pub type NativeFunction = boa_engine::native_function::NativeFunctionPointer;

#[derive(Clone)]
pub struct NativeFunctionBinding {
  name: String,
  length: usize,
  function: NativeFunction,
}

impl NativeFunctionBinding {
  pub fn new(name: impl Into<String>, function: NativeFunction) -> Self {
    Self::with_arity(name, 0, function)
  }

  pub fn with_arity(name: impl Into<String>, arity: usize, function: NativeFunction) -> Self {
    Self {
      name: name.into(),
      length: arity,
      function,
    }
  }

  pub fn name(&self) -> &str {
    &self.name
  }

  pub fn length(&self) -> usize {
    self.length
  }

  pub fn function(&self) -> NativeFunction {
    self.function
  }
}

impl std::fmt::Debug for NativeFunctionBinding {
  fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    formatter
      .debug_struct("NativeFunctionBinding")
      .field("name", &self.name)
      .field("length", &self.length)
      .finish_non_exhaustive()
  }
}
