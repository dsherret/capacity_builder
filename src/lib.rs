use std::borrow::Cow;
use std::cell::Cell;
use std::collections::TryReserveError;
use std::fmt::Write;

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

pub trait StringAppendable {
  fn byte_len(&self) -> usize;
  fn push_to(&self, text: &mut String);
  fn write_to_formatter(
    &self,
    fmt: &mut std::fmt::Formatter<'_>,
  ) -> std::fmt::Result;
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

  #[inline(always)]
  fn write_to_formatter(
    &self,
    fmt: &mut std::fmt::Formatter<'_>,
  ) -> std::fmt::Result {
    fmt.write_str(self)
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

  #[inline(always)]
  fn write_to_formatter(
    &self,
    fmt: &mut std::fmt::Formatter<'_>,
  ) -> std::fmt::Result {
    fmt.write_str(self)
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

  #[inline(always)]
  fn write_to_formatter(
    &self,
    fmt: &mut std::fmt::Formatter<'_>,
  ) -> std::fmt::Result {
    fmt.write_str(self)
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

  #[inline(always)]
  fn write_to_formatter(
    &self,
    fmt: &mut std::fmt::Formatter<'_>,
  ) -> std::fmt::Result {
    fmt.write_char(*self)
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

  #[inline(always)]
  fn write_to_formatter(
    &self,
    fmt: &mut std::fmt::Formatter<'_>,
  ) -> std::fmt::Result {
    (**self).write_to_formatter(fmt)
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

impl StringAppendable for &StringBoxedBuilder<'_> {
  #[inline(always)]
  fn byte_len(&self) -> usize {
    self.capacity()
  }

  #[inline(always)]
  fn push_to(&self, text: &mut String) {
    self.fill_text(text);
  }

  #[inline(always)]
  fn write_to_formatter(
    &self,
    fmt: &mut std::fmt::Formatter<'_>,
  ) -> std::fmt::Result {
    self.fmt(fmt)
  }
}

impl BytesAppendable for &BytesBoxedBuilder<'_> {
  #[inline(always)]
  fn byte_len(&self) -> usize {
    self.capacity()
  }

  #[inline(always)]
  fn push_to(&self, bytes: &mut Vec<u8>) {
    self.fill_bytes(bytes);
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

/// A builder that accepts a boxed closure to build a string.
pub struct StringBoxedBuilder<'a> {
  capacity: Cell<usize>,
  build_fn: Box<dyn Fn(&mut StringBuilder<'a, '_, '_>) + 'a>,
  phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> StringBoxedBuilder<'a> {
  #[inline(always)]
  pub fn new(
    build_fn: Box<dyn Fn(&mut StringBuilder<'a, '_, '_>) + 'a>,
  ) -> Self {
    StringBoxedBuilder {
      build_fn,
      capacity: Cell::new(0),
      phantom: std::marker::PhantomData,
    }
  }

  /// Builds the capacity storing the result or returns
  /// a previously computed capacity.
  pub fn capacity(&self) -> usize {
    let capacity = self.capacity.get();
    if capacity != 0 {
      return capacity;
    }

    let mut builder = StringBuilder {
      mode: Mode::Capacity,
      capacity: 0,
      phantom: std::marker::PhantomData,
    };
    (self.build_fn)(&mut builder);
    self.capacity.set(builder.capacity);
    builder.capacity
  }

  /// Builds the text erroring if the size could not be reserved.
  pub fn build_text(&self) -> Result<String, TryReserveError> {
    let mut text = String::new();
    let mut builder = StringBuilder {
      mode: Mode::Capacity,
      capacity: self.capacity.get(),
      phantom: std::marker::PhantomData,
    };
    if builder.capacity == 0 {
      (self.build_fn)(&mut builder);
      self.capacity.set(builder.capacity);
    }
    text.try_reserve_exact(builder.capacity)?;
    builder.mode = Mode::Text(&mut text);
    (self.build_fn)(&mut builder);
    debug_assert_eq!(builder.capacity, text.len());
    Ok(text)
  }

  pub fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut builder = StringBuilder {
      mode: Mode::Format(fmt),
      capacity: self.capacity.get(),
      phantom: std::marker::PhantomData,
    };
    (self.build_fn)(&mut builder);
    match builder.mode {
      Mode::Format(_) => Ok(()),
      Mode::FormatError(error) => Err(error),
      Mode::Capacity | Mode::Text(_) => unreachable!(),
    }
  }

  fn fill_text(&self, text: &mut String) {
    let mut builder = StringBuilder {
      mode: Mode::Text(text),
      capacity: self.capacity.get(),
      phantom: std::marker::PhantomData,
    };
    (self.build_fn)(&mut builder);
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
  pub fn append(&mut self, value: impl StringAppendable + 'a) {
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

/// A builder that accepts a boxed closure to build a `Vec<u8>`.
pub struct BytesBoxedBuilder<'a> {
  build_fn: Box<dyn Fn(&mut BytesBuilder<'a, '_>) + 'a>,
  capacity: Cell<usize>,
  phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> BytesBoxedBuilder<'a> {
  #[inline(always)]
  pub fn new(build_fn: Box<dyn Fn(&mut BytesBuilder<'a, '_>) + 'a>) -> Self {
    BytesBoxedBuilder {
      build_fn,
      capacity: Cell::new(0),
      phantom: std::marker::PhantomData,
    }
  }

  /// Builds the capacity storing the result or returns
  /// a previously computed capacity.
  pub fn capacity(&self) -> usize {
    let capacity = self.capacity.get();
    if capacity != 0 {
      return capacity;
    }

    let mut builder = BytesBuilder {
      bytes: None,
      capacity: 0,
      phantom: std::marker::PhantomData,
    };
    (self.build_fn)(&mut builder);
    self.capacity.set(builder.capacity);
    builder.capacity
  }

  pub fn build_bytes(&self) -> Result<Vec<u8>, TryReserveError> {
    let mut bytes = Vec::new();
    let mut state = BytesBuilder {
      bytes: None,
      capacity: self.capacity.get(),
      phantom: std::marker::PhantomData,
    };
    if state.capacity == 0 {
      (self.build_fn)(&mut state);
      self.capacity.set(state.capacity);
    }
    bytes.try_reserve_exact(state.capacity)?;
    state.bytes = Some(&mut bytes);
    (self.build_fn)(&mut state);
    debug_assert_eq!(state.capacity, state.bytes.as_ref().unwrap().len());
    Ok(bytes)
  }

  fn fill_bytes(&self, bytes: &mut Vec<u8>) {
    let mut state = BytesBuilder {
      bytes: Some(bytes),
      capacity: self.capacity.get(),
      phantom: std::marker::PhantomData,
    };
    (self.build_fn)(&mut state);
  }
}

#[cfg(test)]
mod test {
  use crate::BytesBoxedBuilder;
  use crate::BytesBuilder;
  use crate::StringAppendable;
  use crate::StringBoxedBuilder;
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
  fn string_builder_appended() {
    let other_builder = StringBoxedBuilder::new(Box::new(|builder| {
      builder.append("Hello");
    }));

    let text = StringBuilder::build(|builder| {
      builder.append(&other_builder);
      builder.append(" there!");
    });

    assert_eq!(text.unwrap(), "Hello there!");
  }

  #[test]
  fn bytes_builder_appended() {
    let other_builder = BytesBoxedBuilder::new(Box::new(|builder| {
      builder.append("Hello");
    }));

    let bytes = BytesBuilder::build(|builder| {
      builder.append(&other_builder);
      builder.append(" there!");
    });

    assert_eq!(bytes.unwrap(), "Hello there!".as_bytes());
  }

  #[test]
  fn formatter() {
    struct MyStruct;

    impl std::fmt::Display for MyStruct {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let other_builder = StringBoxedBuilder::new(Box::new(|builder| {
          builder.append("Hello");
        }));

        other_builder.fmt(f)?;

        StringBuilder::fmt(f, |builder| {
          builder.append(&other_builder);
          builder.append(" there!");
        })
      }
    }

    assert_eq!(format!("{}", MyStruct), "HelloHello there!");
  }

  #[test]
  fn formatter_error() {
    struct AppendableError;

    impl StringAppendable for AppendableError {
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
        let builder = StringBoxedBuilder::new(Box::new(|builder| {
          builder.append("Will show");
          builder.append(AppendableError);
          builder.append("This won't");
        }));

        let result = builder.fmt(f);
        assert!(result.is_err());

        Ok(())
      }
    }

    assert_eq!(format!("{}", MyStruct), "Will show");
  }
}
