use capacity_builder::StringBuilder;

fn main() {
  // Run registered benchmarks.
  divan::main();
}

mod strings {
  use super::*;

  #[divan::bench]
  fn string_builder() -> usize {
    StringBuilder::<String>::build(|builder| {
      builder.append("Hello, ");
      builder.append("world!");
      builder.append("testing ");
    })
    .unwrap()
    .len()
  }

  #[divan::bench]
  fn text_new() -> usize {
    let mut text = String::new();
    text.push_str("Hello, ");
    text.push_str("world!");
    text.push_str(" testing ");
    text.shrink_to_fit();
    text.len()
  }
}

mod numbers {
  use super::*;
  #[divan::bench]
  fn string_builder() -> usize {
    StringBuilder::<String>::build(|builder| {
      for i in 0..1000 {
        builder.append(i);
      }
    })
    .unwrap()
    .len()
  }

  #[divan::bench]
  fn text_new() -> usize {
    let mut text = String::new();
    for i in 0..1000 {
      text.push_str(&i.to_string());
    }
    text.shrink_to_fit();
    text.len()
  }
}
