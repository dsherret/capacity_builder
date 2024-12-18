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
