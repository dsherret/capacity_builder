use crate::StringType;
use crate::StringTypeMut;

impl StringType for ecow::EcoString {
  type MutType = ecow::EcoString;

  #[inline(always)]
  fn with_capacity(
    size: usize,
  ) -> Result<Self::MutType, std::collections::TryReserveError> {
    Ok(ecow::EcoString::with_capacity(size))
  }

  #[inline(always)]
  fn from_mut(inner: Self::MutType) -> Self {
    inner
  }
}

impl StringTypeMut for ecow::EcoString {
  fn push(&mut self, c: char) {
    self.push(c);
  }

  fn push_str(&mut self, str: &str) {
    self.push_str(str);
  }

  fn len(&self) -> usize {
    self.len()
  }
}

#[cfg(test)]
mod test {
  use crate::StringBuilder;

  #[test]
  fn builds() {
    let text = StringBuilder::<ecow::EcoString>::build(|builder| {
      builder.append("Hello");
      builder.append(" there!");
    })
    .unwrap();
    assert_eq!(text, "Hello there!");
  }
}
