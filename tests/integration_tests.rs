use capacity_builder::BytesAppendable;
use capacity_builder::BytesBuilder;
use capacity_builder::FastDisplay;
use capacity_builder::StringBuildable;
use capacity_builder::StringBuilder;

#[test]
fn string_buildable() {
  #[derive(FastDisplay)]
  struct MyStruct;

  impl StringBuildable for MyStruct {
    fn string_build_with<'a>(
      &'a self,
      builder: &mut StringBuilder<'a, '_, '_>,
    ) {
      builder.append("Hello");
      builder.append(" there!");
    }
  }

  let text = StringBuilder::build(|builder| {
    builder.append(&MyStruct);
  })
  .unwrap();
  assert_eq!(text, "Hello there!");
}

#[test]
fn bytes_appendable() {
  struct MyStruct;

  impl<'a> BytesAppendable<'a> for &'a MyStruct {
    fn append_to_builder(self, builder: &mut BytesBuilder<'a, '_>) {
      builder.append("Hello");
      builder.append(" there!");
    }
  }

  let bytes = BytesBuilder::build(|builder| {
    builder.append(&MyStruct);
  })
  .unwrap();
  assert_eq!(bytes, b"Hello there!");
}
