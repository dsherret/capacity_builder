use hipstr::HipStr;

use crate::StringType;
use crate::StringTypeMut;

impl StringType for HipStr<'static> {
  type MutType = HipStr<'static>;

  #[inline(always)]
  fn with_capacity(
    size: usize,
  ) -> Result<Self::MutType, std::collections::TryReserveError> {
    Ok(HipStr::with_capacity(size))
  }

  #[inline(always)]
  fn from_mut(inner: Self::MutType) -> Self {
    inner
  }
}

impl StringTypeMut for HipStr<'static> {
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
    let text = StringBuilder::<hipstr::HipStr>::build(|builder| {
      builder.append("Hello");
      builder.append(" there!");
    })
    .unwrap();
    assert_eq!(text, "Hello there!");
  }
}
