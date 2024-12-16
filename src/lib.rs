use std::borrow::Cow;
use std::collections::TryReserveError;

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

      impl StringAppendable for $t {
        fn byte_len(&self) -> usize {
          count_digits!(*self)
        }

        fn push_to(&self, text: &mut String) {
          let value = *self;
          // no need to reuse buffers as per the documentation
          // and as found in my benchmarks
          let mut buffer = itoa::Buffer::new();
          let s = buffer.format(value);
          text.push_str(s);
        }
      }
    )*
  };
}

pub trait StringAppendable {
  fn byte_len(&self) -> usize;
  fn push_to(&self, text: &mut String);
}

pub trait BytesAppendable {
  fn byte_len(&self) -> usize;
  fn push_to(&self, bytes: &mut Vec<u8>);
}

pub trait EndianBytesAppendable {
  fn byte_len(&self) -> usize;
  fn push_le_to(&self, bytes: &mut Vec<u8>);
  fn push_be_to(&self, bytes: &mut Vec<u8>);
}

impl StringAppendable for &str {
  #[inline(always)]
  fn byte_len(&self) -> usize {
    self.len()
  }

  #[inline(always)]
  fn push_to(&self, text: &mut String) {
    text.push_str(self);
  }
}

impl BytesAppendable for &str {
  #[inline(always)]
  fn byte_len(&self) -> usize {
    self.len()
  }

  #[inline(always)]
  fn push_to(&self, bytes: &mut Vec<u8>) {
    bytes.extend_from_slice(self.as_bytes());
  }
}

impl StringAppendable for &String {
  #[inline(always)]
  fn byte_len(&self) -> usize {
    self.len()
  }

  #[inline(always)]
  fn push_to(&self, text: &mut String) {
    text.push_str(self);
  }
}

impl BytesAppendable for &String {
  #[inline(always)]
  fn byte_len(&self) -> usize {
    self.len()
  }

  #[inline(always)]
  fn push_to(&self, bytes: &mut Vec<u8>) {
    bytes.extend_from_slice(self.as_bytes());
  }
}

impl<'a> StringAppendable for &'a Cow<'a, str> {
  #[inline(always)]
  fn byte_len(&self) -> usize {
    self.len()
  }

  #[inline(always)]
  fn push_to(&self, text: &mut String) {
    text.push_str(self);
  }
}

impl<'a> BytesAppendable for &'a Cow<'a, str> {
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

impl StringAppendable for char {
  #[inline(always)]
  fn byte_len(&self) -> usize {
    self.len_utf8()
  }

  #[inline(always)]
  fn push_to(&self, text: &mut String) {
    text.push(*self);
  }
}

impl BytesAppendable for char {
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

impl<T: StringAppendable> StringAppendable for Option<T> {
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
}

impl<T: BytesAppendable> BytesAppendable for Option<T> {
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

impl<T: StringAppendable + ?Sized> StringAppendable for &T {
  #[inline(always)]
  fn byte_len(&self) -> usize {
    (**self).byte_len()
  }

  #[inline(always)]
  fn push_to(&self, text: &mut String) {
    (**self).push_to(text)
  }
}

impl<T: BytesAppendable + ?Sized> BytesAppendable for &T {
  #[inline(always)]
  fn byte_len(&self) -> usize {
    (**self).byte_len()
  }

  #[inline(always)]
  fn push_to(&self, bytes: &mut Vec<u8>) {
    (**self).push_to(bytes)
  }
}

impl BytesAppendable for &Vec<u8> {
  #[inline(always)]
  fn byte_len(&self) -> usize {
    self.len()
  }

  #[inline(always)]
  fn push_to(&self, bytes: &mut Vec<u8>) {
    bytes.extend_from_slice(self)
  }
}

impl BytesAppendable for u8 {
  #[inline(always)]
  fn byte_len(&self) -> usize {
    1
  }

  #[inline(always)]
  fn push_to(&self, bytes: &mut Vec<u8>) {
    bytes.push(*self)
  }
}

impl BytesAppendable for [u8] {
  #[inline(always)]
  fn byte_len(&self) -> usize {
    self.len()
  }

  #[inline(always)]
  fn push_to(&self, bytes: &mut Vec<u8>) {
    bytes.extend_from_slice(self)
  }
}

impl<const N: usize> BytesAppendable for [u8; N] {
  #[inline(always)]
  fn byte_len(&self) -> usize {
    N
  }

  #[inline(always)]
  fn push_to(&self, bytes: &mut Vec<u8>) {
    bytes.extend_from_slice(self);
  }
}

pub struct StringBuilder<'a> {
  capacity: usize,
  text: Option<String>,
  phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> StringBuilder<'a> {
  #[inline(always)]
  pub fn build(
    build: impl Fn(&mut StringBuilder<'a>),
  ) -> Result<String, TryReserveError> {
    let mut builder = StringBuilder {
      capacity: 0,
      text: None,
      phantom: std::marker::PhantomData,
    };
    build(&mut builder);
    let mut text = String::new();
    text.try_reserve_exact(builder.capacity)?;
    builder.text = Some(text);
    build(&mut builder);
    debug_assert_eq!(builder.capacity, builder.text.as_ref().unwrap().len());
    Ok(builder.text.unwrap())
  }

  /// Gets the current length of the builder.
  ///
  /// On the first pass this will be the current calculated capacity and
  /// on the second pass it will be the current length of the string.
  #[allow(clippy::len_without_is_empty)]
  pub fn len(&self) -> usize {
    self.text.as_ref().map(|t| t.len()).unwrap_or(self.capacity)
  }

  #[inline(always)]
  pub fn append(&mut self, value: impl StringAppendable + 'a) {
    match &mut self.text {
      Some(t) => value.push_to(t),
      None => self.capacity += value.byte_len(),
    }
  }
}

pub struct BytesBuilder<'a> {
  capacity: usize,
  bytes: Option<Vec<u8>>,
  phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> BytesBuilder<'a> {
  #[inline(always)]
  pub fn build(
    build: impl Fn(&mut BytesBuilder<'a>),
  ) -> Result<Vec<u8>, TryReserveError> {
    let mut builder = BytesBuilder {
      capacity: 0,
      bytes: None,
      phantom: std::marker::PhantomData,
    };
    build(&mut builder);
    let mut bytes = Vec::new();
    bytes.try_reserve_exact(builder.capacity)?;
    builder.bytes = Some(bytes);
    build(&mut builder);
    debug_assert_eq!(builder.capacity, builder.bytes.as_ref().unwrap().len());
    Ok(builder.bytes.unwrap())
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
  pub fn append(&mut self, value: impl BytesAppendable + 'a) {
    match &mut self.bytes {
      Some(b) => value.push_to(b),
      None => self.capacity += value.byte_len(),
    }
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
}
