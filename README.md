# `capacity_builder`

Builders where the code to calculate the capacity is the same as the code to
write what's being built.

## Overview

Sometimes you have some complex code that would be a bit of a pain to calculate
the capacity of or could risk easily getting out of sync with the
implementation. This crate makes keeping it in sync easier because it's the same
code.

```rs
use capacity_builder::StringBuilder;

let text = StringBuilder::build(|builder| {
  for (i, import_module) in import_modules.iter().enumerate() {
    builder.append("// ");
    builder.append(i);
    builder.append(" import\n");
    builder.append("import \"");
    builder.append(import_module);
    builder.append("\";\n");
  }
})?;
```

Behind the scenes it runs the closure once to compute the capacity and a second
time to write the string.

## `StringBuilder` - Use with `std::fmt::Display`

It's possible to use this with the `Display` trait by calling
`StringBuilder::fmt`. This can be useful when you want to re-use the code for
building a string with the code for the `Display` trait.

```rs
impl Version {
  // ...

  // ok because this to_string() is about 20% faster than reusing
  // the Display impl
  #[allow(clippy::inherent_to_string_shadow_display)]
  pub fn to_string(&self) -> String {
    capacity_builder::StringBuilder::build(|builder| {
      build_version_to_string(self, builder);
    })
    .unwrap()
  }
}

impl fmt::Display for Version {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    capacity_builder::StringBuilder::fmt(f, |builder| {
      build_version_to_string(self, builder);
    })
  }
}

fn build_version_to_string<'a>(
  version: &'a Version,
  builder: &mut capacity_builder::StringBuilder<'a, '_, '_>,
) {
  builder.append(version.major);
  builder.append('.');
  builder.append(version.minor);
  builder.append('.');
  builder.append(version.patch);
  if !version.pre.is_empty() {
    builder.append('-');
    for (i, part) in version.pre.iter().enumerate() {
      if i > 0 {
        builder.append('.');
      }
      builder.append(part);
    }
  }
  if !version.build.is_empty() {
    builder.append('+');
    for (i, part) in version.build.iter().enumerate() {
      if i > 0 {
        builder.append('.');
      }
      builder.append(part);
    }
  }
}
```

You may have noticed that no errors are necessary to surface. This is because
generally errors when formatting are really rare and if an error is encountered
it will store it to surface at the end and the rest of the `append` statements
stop formatting.

## Boxed Builder

```rs
use capacity_builder::StringBoxedBuilder;

let builder = StringBoxedBuilder::new(|builder| {
  builder.append(123);
  builder.append(',');
  builder.append(456);
});

assert_eq!(builder.build_capacity(), 7);
// memoized
assert_eq!(builder.build_capacity(), 7);
// not memoized, but uses previously computed capacity
assert_eq!(builder.build_text(), "123,456");
```

Note that a boxed builder will be slightly slower than just using the
`StringBuilder::build` function.

### Composing boxed builders

It's possible to compose boxed builders by appending them to another builder:

```rs
let other_builder = StringBoxedBuilder::new(|builder| {
  builder.append("Hello");
});

let text = StringBuilder::build(|builder| {
  builder.append(&other_builder);
  builder.append(" there!");
});

assert_eq!(text.unwrap(), "Hello there!");
```

## Features

1. The builder prevents adding owned data—only references.
   - This helps to prevent accidentally allocating data multiple times in the
     closure.
1. Errors when capacity cannot be reserved.
1. For the string builder, types other than references can be provided.
   - Numbers get written with the [itoa](https://crates.io/crates/itoa) crate.

## Tips

- Do any necessary allocations before running the closure.
- Measure before and after using this crate to ensure you're not slower.
