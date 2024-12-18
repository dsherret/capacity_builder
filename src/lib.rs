use std::borrow::Cow;
use std::collections::TryReserveError;
use std::fmt::Write;

pub use capacity_builder_macros::FastDisplay;

macro_rules! count_digits {
  ($value:expr) => {{
    let mut value = $value;
    if value == 0 {
      1
    } else {
      let mut count = 0;
      while value > 0 {
        value /= 10;
        count += 1;
      }
      count
    }
  }};
}

macro_rules! impl_appendable_for_int {
  ($($t:ty),*) => {
    $(
      impl EndianBytesAppendable for $t {
        fn byte_len(&self) -> usize {
          std::mem::size_of::<$t>()
        }

        fn push_le_to(&self, bytes: &mut Vec<u8>) {
          bytes.extend_from_slice(&self.to_le_bytes());
        }

        fn push_be_to(&self, bytes: &mut Vec<u8>) {
          bytes.extend_from_slice(&self.to_be_bytes());
        }
      }

      impl StringAppendableValue for $t {
        fn byte_len(&self) -> usize {
          count_digits!(*self)
        }

        fn push_to(&self, text: &mut String) {
          // no need to reuse buffers as per the documentation
          // and as found in my benchmarks
          let mut buffer = itoa::Buffer::new();
          let s = buffer.format(*self);
          text.push_str(s);
        }

        #[inline(always)]
        fn write_to_formatter(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
          let mut buffer = itoa::Buffer::new();
          let s = buffer.format(*self);
          fmt.write_str(s)
        }
      }
    )*
  };
}

pub trait StringAppendable<'a> {
  fn append_to_builder(self, builder: &mut StringBuilder<'a, '_, '_>);
}

impl<'a, T> StringAppendable<'a> for T
where
  T: StringAppendableValue,
{
  fn append_to_builder(self, builder: &mut StringBuilder<'a, '_, '_>) {
    builder.append_value(self);
  }
}

pub trait StringBuildable {
  fn string_build_with<'a>(&'a self, builder: &mut StringBuilder<'a, '_, '_>);
}

pub trait StringAppendableValue {
  fn byte_len(&self) -> usize;
  fn push_to(&self, text: &mut String);
  fn write_to_formatter(
    &self,
    fmt: &mut std::fmt::Formatter<'_>,
  ) -> std::fmt::Result;
}

pub trait BytesAppendable<'a> {
  fn append_to_builder(self, builder: &mut BytesBuilder<'a, '_>);
}

pub trait BytesAppendableValue {
  fn byte_len(&self) -> usize;
  fn push_to(&self, bytes: &mut Vec<u8>);
}

impl<'a, T: BytesAppendableValue> BytesAppendable<'a> for T {
  fn append_to_builder(self, builder: &mut BytesBuilder<'a, '_>) {
    match &mut builder.bytes {
      Some(b) => self.push_to(b),
      None => builder.capacity += self.byte_len(),
    }
  }
}

pub trait EndianBytesAppendable {
  fn byte_len(&self) -> usize;
  fn push_le_to(&self, bytes: &mut Vec<u8>);
  fn push_be_to(&self, bytes: &mut Vec<u8>);
}

impl StringAppendableValue for &str {
  #[inline(always)]
  fn byte_len(&self) -> usize {
    self.len()
  }

  #[inline(always)]
  fn push_to(&self, text: &mut String) {
    text.push_str(self);
  }

  #[inline(always)]
  fn write_to_formatter(
    &self,
    fmt: &mut std::fmt::Formatter<'_>,
  ) -> std::fmt::Result {
    fmt.write_str(self)
  }
}

impl BytesAppendableValue for &str {
  #[inline(always)]
  fn byte_len(&self) -> usize {
    self.len()
  }

  #[inline(always)]
  fn push_to(&self, bytes: &mut Vec<u8>) {
    bytes.extend_from_slice(self.as_bytes());
  }
}

impl StringAppendableValue for &String {
  #[inline(always)]
  fn byte_len(&self) -> usize {
    self.len()
  }

  #[inline(always)]
  fn push_to(&self, text: &mut String) {
    text.push_str(self);
  }

  #[inline(always)]
  fn write_to_formatter(
    &self,
    fmt: &mut std::fmt::Formatter<'_>,
  ) -> std::fmt::Result {
    fmt.write_str(self)
  }
}

impl BytesAppendableValue for &String {
  #[inline(always)]
  fn byte_len(&self) -> usize {
    self.len()
  }

  #[inline(always)]
  fn push_to(&self, bytes: &mut Vec<u8>) {
    bytes.extend_from_slice(self.as_bytes());
  }
}

impl<'a> StringAppendableValue for &'a Cow<'a, str> {
  #[inline(always)]
  fn byte_len(&self) -> usize {
    self.len()
  }

  #[inline(always)]
  fn push_to(&self, text: &mut String) {
    text.push_str(self);
  }

  #[inline(always)]
  fn write_to_formatter(
    &self,
    fmt: &mut std::fmt::Formatter<'_>,
  ) -> std::fmt::Result {
    fmt.write_str(self)
  }
}

impl<'a> BytesAppendableValue for &'a Cow<'a, str> {
  #[inline(always)]
  fn byte_len(&self) -> usize {
    self.len()
  }

  #[inline(always)]
  fn push_to(&self, bytes: &mut Vec<u8>) {
    bytes.extend(self.as_bytes());
  }
}

impl_appendable_for_int!(
  i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize
);

impl StringAppendableValue for char {
  #[inline(always)]
  fn byte_len(&self) -> usize {
    self.len_utf8()
  }

  #[inline(always)]
  fn push_to(&self, text: &mut String) {
    text.push(*self);
  }

  #[inline(always)]
  fn write_to_formatter(
    &self,
    fmt: &mut std::fmt::Formatter<'_>,
  ) -> std::fmt::Result {
    fmt.write_char(*self)
  }
}

impl BytesAppendableValue for char {
  #[inline(always)]
  fn byte_len(&self) -> usize {
    self.len_utf8()
  }

  fn push_to(&self, bytes: &mut Vec<u8>) {
    let mut buffer = [0; 4];
    let encoded = self.encode_utf8(&mut buffer);
    bytes.extend_from_slice(encoded.as_bytes())
  }
}

impl<T: StringAppendableValue> StringAppendableValue for Option<T> {
  #[inline(always)]
  fn byte_len(&self) -> usize {
    match self {
      Some(value) => value.byte_len(),
      None => 0,
    }
  }

  #[inline(always)]
  fn push_to(&self, text: &mut String) {
    if let Some(value) = self {
      value.push_to(text);
    }
  }

  #[inline(always)]
  fn write_to_formatter(
    &self,
    fmt: &mut std::fmt::Formatter<'_>,
  ) -> std::fmt::Result {
    if let Some(value) = self {
      value.write_to_formatter(fmt)
    } else {
      Ok(())
    }
  }
}

impl<T: BytesAppendableValue> BytesAppendableValue for Option<T> {
  #[inline(always)]
  fn byte_len(&self) -> usize {
    match self {
      Some(value) => value.byte_len(),
      None => 0,
    }
  }

  #[inline(always)]
  fn push_to(&self, bytes: &mut Vec<u8>) {
    if let Some(value) = self {
      value.push_to(bytes);
    }
  }
}

impl<T: StringAppendableValue + ?Sized> StringAppendableValue for &T {
  #[inline(always)]
  fn byte_len(&self) -> usize {
    (**self).byte_len()
  }

  #[inline(always)]
  fn push_to(&self, text: &mut String) {
    (**self).push_to(text)
  }

  #[inline(always)]
  fn write_to_formatter(
    &self,
    fmt: &mut std::fmt::Formatter<'_>,
  ) -> std::fmt::Result {
    (**self).write_to_formatter(fmt)
  }
}

impl<T: BytesAppendableValue + ?Sized> BytesAppendableValue for &T {
  #[inline(always)]
  fn byte_len(&self) -> usize {
    (**self).byte_len()
  }

  #[inline(always)]
  fn push_to(&self, bytes: &mut Vec<u8>) {
    (**self).push_to(bytes)
  }
}

impl BytesAppendableValue for &Vec<u8> {
  #[inline(always)]
  fn byte_len(&self) -> usize {
    self.len()
  }

  #[inline(always)]
  fn push_to(&self, bytes: &mut Vec<u8>) {
    bytes.extend_from_slice(self)
  }
}

impl BytesAppendableValue for u8 {
  #[inline(always)]
  fn byte_len(&self) -> usize {
    1
  }

  #[inline(always)]
  fn push_to(&self, bytes: &mut Vec<u8>) {
    bytes.push(*self)
  }
}

impl BytesAppendableValue for [u8] {
  #[inline(always)]
  fn byte_len(&self) -> usize {
    self.len()
  }

  #[inline(always)]
  fn push_to(&self, bytes: &mut Vec<u8>) {
    bytes.extend_from_slice(self)
  }
}

impl<const N: usize> BytesAppendableValue for [u8; N] {
  #[inline(always)]
  fn byte_len(&self) -> usize {
    N
  }

  #[inline(always)]
  fn push_to(&self, bytes: &mut Vec<u8>) {
    bytes.extend_from_slice(self);
  }
}

enum Mode<'a, 'b> {
  Capacity,
  Text(&'a mut String),
  Format(&'a mut std::fmt::Formatter<'b>),
  FormatError(std::fmt::Error),
}

pub struct StringBuilder<'a, 'b, 'c> {
  capacity: usize,
  mode: Mode<'b, 'c>,
  phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> StringBuilder<'a, '_, '_> {
  #[inline(always)]
  pub fn build(
    build: impl Fn(&mut StringBuilder<'a, '_, '_>),
  ) -> Result<String, TryReserveError> {
    let mut state = StringBuilder {
      mode: Mode::Capacity,
      capacity: 0,
      phantom: std::marker::PhantomData,
    };
    build(&mut state);
    let mut text = String::new();
    text.try_reserve_exact(state.capacity)?;
    state.mode = Mode::Text(&mut text);
    build(&mut state);
    debug_assert_eq!(state.capacity, text.len());
    Ok(text)
  }

  /// Formats the string using the provided formatter.
  ///
  /// If an error occurs, the error is stored and surfaced
  /// at the end. The remaining `append` calls are then ignored.
  #[inline(always)]
  pub fn fmt(
    fmt: &mut std::fmt::Formatter<'_>,
    build: impl FnOnce(&mut StringBuilder<'a, '_, '_>),
  ) -> std::fmt::Result {
    let mut state = StringBuilder {
      mode: Mode::Format(fmt),
      capacity: 0,
      phantom: std::marker::PhantomData,
    };
    build(&mut state);
    match state.mode {
      Mode::Format(_) => Ok(()),
      Mode::FormatError(error) => Err(error),
      Mode::Capacity | Mode::Text(_) => unreachable!(),
    }
  }

  /// Gets the current length of the builder.
  ///
  /// On the first pass this will be the current calculated capacity and
  /// on the second pass it will be the current length of the string.
  #[allow(clippy::len_without_is_empty)]
  pub fn len(&self) -> usize {
    match &self.mode {
      Mode::Text(t) => t.len(),
      Mode::Capacity | Mode::Format(_) | Mode::FormatError(_) => self.capacity,
    }
  }

  #[inline(always)]
  pub fn append(&mut self, value: impl StringAppendable<'a> + 'a) {
    value.append_to_builder(self);
  }

  fn append_value(&mut self, value: impl StringAppendableValue) {
    match &mut self.mode {
      Mode::Text(t) => value.push_to(t),
      Mode::Capacity => self.capacity += value.byte_len(),
      Mode::Format(formatter) => {
        let result = value.write_to_formatter(formatter);
        if let Err(e) = result {
          // this is very rare, so if it happens we transition
          // to an error state, storing the error to be surfaced
          // later and don't bother formatting the remaining bytes
          self.mode = Mode::FormatError(e);
        }
        self.capacity += value.byte_len();
      }
      Mode::FormatError(_) => {
        // keep setting the capacity in case the remaining
        // code relies on this
        self.capacity += value.byte_len();
      }
    }
  }
}

pub struct BytesBuilder<'a, 'b> {
  capacity: usize,
  bytes: Option<&'b mut Vec<u8>>,
  phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> BytesBuilder<'a, '_> {
  #[inline(always)]
  pub fn build(
    build: impl Fn(&mut BytesBuilder<'a, '_>),
  ) -> Result<Vec<u8>, TryReserveError> {
    let mut builder = BytesBuilder {
      bytes: None,
      capacity: 0,
      phantom: std::marker::PhantomData,
    };
    build(&mut builder);
    let mut bytes = Vec::new();
    bytes.try_reserve_exact(builder.capacity)?;
    builder.bytes = Some(&mut bytes);
    build(&mut builder);
    debug_assert_eq!(builder.capacity, builder.bytes.as_ref().unwrap().len());
    Ok(bytes)
  }

  /// Gets the current length of the builder.
  ///
  /// On the first pass this will be the current calculated capacity and
  /// on the second pass it will be the current length of the bytes.
  #[allow(clippy::len_without_is_empty)]
  pub fn len(&self) -> usize {
    self
      .bytes
      .as_ref()
      .map(|t| t.len())
      .unwrap_or(self.capacity)
  }

  #[inline(always)]
  pub fn append(&mut self, value: impl BytesAppendable<'a> + 'a) {
    value.append_to_builder(self);
  }

  /// Appends a number in big-endian byte order.
  ///
  /// WARNING: Rust defaults to i32 for integer literals. It's probably
  /// best to always specify the type of number.
  #[inline(always)]
  pub fn append_be<T: EndianBytesAppendable + 'a>(&mut self, value: T) {
    match &mut self.bytes {
      Some(b) => value.push_be_to(b),
      None => self.capacity += value.byte_len(),
    }
  }

  /// Appends a number in little-endian byte order.
  ///
  /// WARNING: Rust defaults to i32 for integer literals. It's probably
  /// best to always specify the type of number.
  #[inline(always)]
  pub fn append_le<T: EndianBytesAppendable + 'a>(&mut self, value: T) {
    match &mut self.bytes {
      Some(b) => value.push_le_to(b),
      None => self.capacity += value.byte_len(),
    }
  }
}

#[cfg(test)]
mod test {
  use crate::BytesBuilder;
  use crate::StringAppendableValue;
  use crate::StringBuilder;

  #[test]
  fn bytes_builder_be_and_le() {
    let bytes = BytesBuilder::build(|builder| {
      builder.append_be(6i32);
      builder.append_le(8i32);
    })
    .unwrap();
    assert_eq!(bytes, vec![0, 0, 0, 6, 8, 0, 0, 0]);
  }

  #[test]
  fn bytes_builder() {
    let bytes = BytesBuilder::build(|builder| {
      builder.append("Hello, ");
      assert_eq!(builder.len(), 7);
      builder.append("world!");
      assert_eq!(builder.len(), 13);
      builder.append("testing ");
      assert_eq!(builder.len(), 21);
    })
    .unwrap();
    assert_eq!(String::from_utf8(bytes).unwrap(), "Hello, world!testing ");
  }

  #[test]
  fn formatter() {
    struct MyStruct;

    impl std::fmt::Display for MyStruct {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        StringBuilder::fmt(f, |builder| {
          builder.append("Hello there!");
        })
      }
    }

    assert_eq!(format!("{}", MyStruct), "Hello there!");
  }

  #[test]
  fn formatter_error() {
    struct AppendableError;

    impl StringAppendableValue for AppendableError {
      fn byte_len(&self) -> usize {
        1
      }

      fn push_to(&self, _text: &mut String) {
        unreachable!();
      }

      fn write_to_formatter(
        &self,
        _fmt: &mut std::fmt::Formatter<'_>,
      ) -> std::fmt::Result {
        std::fmt::Result::Err(std::fmt::Error)
      }
    }

    struct MyStruct;

    impl std::fmt::Display for MyStruct {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let result = StringBuilder::fmt(f, |builder| {
          builder.append("Will show");
          builder.append(AppendableError);
          builder.append("This won't");
        });
        assert!(result.is_err());

        Ok(())
      }
    }

    assert_eq!(format!("{}", MyStruct), "Will show");
  }
}
