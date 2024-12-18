use capacity_builder::BytesAppendable;
use capacity_builder::BytesBuilder;
use capacity_builder::BytesType;
use capacity_builder::FastDisplay;
use capacity_builder::StringAppendable;
use capacity_builder::StringBuilder;
use capacity_builder::StringType;

#[derive(FastDisplay)]
struct MyStruct;

impl<'a> StringAppendable<'a> for &'a MyStruct {
  fn append_to_builder<TString: StringType>(
    self,
    builder: &mut StringBuilder<'a, TString>,
  ) {
    builder.append("Hello");
    builder.append(" there!");
  }
}

#[test]
fn string_buildable() {
  let text = StringBuilder::<String>::build(|builder| {
    builder.append(&MyStruct);
  })
  .unwrap();
  assert_eq!(text, "Hello there!");
  assert_eq!(format!("{}", MyStruct), "Hello there!");
  assert_eq!(MyStruct.to_string(), "Hello there!");
}

#[test]
fn bytes_appendable() {
  struct MyStruct;

  impl<'a> BytesAppendable<'a> for &'a MyStruct {
    fn append_to_builder<TBytes: BytesType>(
      self,
      builder: &mut BytesBuilder<'a, TBytes>,
    ) {
      builder.append("Hello");
      builder.append(" there!");
    }
  }

  let bytes = BytesBuilder::<Vec<u8>>::build(|builder| {
    builder.append(&MyStruct);
  })
  .unwrap();
  assert_eq!(bytes, b"Hello there!");
}

#[test]
fn box_str() {
  let boxed_str = " there".to_string().into_boxed_str();
  let text = StringBuilder::<Box<str>>::build(|builder| {
    builder.append("hi");
    builder.append(&boxed_str);
  })
  .unwrap();
  assert_eq!(text, "hi there".to_string().into_boxed_str());
}

#[test]
fn box_slice() {
  let box_slice = " there".as_bytes().to_vec().into_boxed_slice();
  let bytes = BytesBuilder::<Box<[u8]>>::build(|builder| {
    builder.append("hi");
    builder.append(&box_slice);
  })
  .unwrap();
  assert_eq!(bytes, "hi there".as_bytes().to_vec().into_boxed_slice());
}

#[cfg(feature = "ecow")]
#[test]
fn to_string_helpers_ecow() {
  let text = MyStruct.to_string_ecow();
  assert_eq!(text, "Hello there!");
}

#[cfg(feature = "hipstr")]
#[test]
fn to_string_helpers_hipstr() {
  let text = MyStruct.to_string_hipstr();
  assert_eq!(text, "Hello there!");
}
