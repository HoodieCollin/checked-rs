# checked-rs

`checked-rs` (referred to as `checked`) is a Rust library that includes generic types providing semantics for clamped unsigned integers and a general-purpose type that associates a data type with a validation type. This library was extracted from a larger side-project to make it generally available and showcase Rust skills and knowledge.

## Installation

The `checked` library is compatible with `rustc 1.79.0-nightly (a07f3eb43 2024-04-11)` but does not use any opt-in language features. To install `checked-rs`, add the following to your `Cargo.toml`:

```toml
[dependencies]
checked-rs = "0.1.0"
```

## Panicking vs Saturating

Behavior types are provide that configure the types to follow either panicking or saturating semantics when an underflow or overflow occurs. The `clamped` attribute macro defaults to panicking behavior if it is not otherwise specified.

## Usage

### Hard Clamp (Direct Interaction)
```rust
use checked::HardClamp;
use checked::behaviors::Saturating;

let mut clamp = HardClamp::<u8, Saturating, 0, 10>::new(5)?;

assert_eq!(clamp.get(), 5);

clamp += 5;

assert_eq!(clamp.get(), 10);

clamp -= 15;

assert_eq!(clamp.get(), 0);

clamp += 20;

assert_eq!(clamp.get(), 10);
```

### Hard Clamp Guard
```rust
use checked::{HardClamp, GuardState};
use checked::behaviors::Saturating;

let mut clamp = HardClamp::<u8, Saturating, 0, 10>::new(5)?;

assert_eq!(clamp.get(), 5);

let mut g = clamp.modify();

assert_eq!(g.check(), GuardState::Unchanged);

*g = 10;

assert_eq!(g.check(), GuardState::Changed);

g.commit()?;

assert_eq!(clamp.get(), 10);

let mut g = clamp.modify();

assert_eq!(g.check(), GuardState::Unchanged);

*g = 15;

assert_eq!(g.check(), GuardState::Changed);

assert!(g.commit().is_err());

assert_eq!(clamp.get(), 10);
```

### Soft Clamp
```rust
use checked::SoftClamp;
use checked::behaviors::Saturating;

let mut clamp = SoftClamp::<u8, Saturating, 0, 10>::new(5);
assert_eq!(*clamp, 5);
assert!(clamp.is_valid());

clamp += 5;
assert_eq!(*clamp, 10);
assert!(clamp.is_valid());

clamp -= 15;
assert_eq!(*clamp, 0);
assert!(clamp.is_valid());

*clamp = 30;
assert_eq!(*clamp, 30);
assert_eq!(clamp.is_valid(), false);
```

### View
```rust
use checked::{View, Validator, Result};

struct CheckedIntValidator;

impl Validator for CheckedIntValidator {
    type Item = i32;

    fn validate(item: &Self::Item) -> Result<()> {
        if *item < 0 {
            Err(anyhow::anyhow!("Value must be positive"))
        } else if *item % 2 == 0 && *item != 0 && *item <= 10 {
            Err(anyhow::anyhow!("Value must be odd, or zero, or greater than 10"))
        } else if *item == 7 {
            Err(anyhow::anyhow!("Value must not be 7"))
        } else {
            Ok(())
        }
    }
}

let mut item = View::with_validator(0, CheckedIntValidator);

assert_eq!(*item, 0);
assert!(item.is_valid());

*item = 1;
assert_eq!(*item, 1);
assert!(item.check().is_ok());

*item = -1;
assert_eq!(*item, -1);
assert!(item.check().is_err());

*item = 12;
assert_eq!(*item, 12);
assert!(item.check().is_ok());

*item = 7;
assert_eq!(*item, 7);

let item = match item.try_unwrap() {
    Ok(_) => panic!("Expected error"),
    Err(item) => {
        assert_eq!(*item, 7);
        item
    }
};

item.cancel();
```

### `#[clamped(..)]` Attribute Macro
```rust
use checked::clamped;

#[clamped(u8; default = 1; behavior = Panicking)]
#[derive(Debug, Clone, Copy)]
pub enum Example {
    #[eq(0)]
    Nil,
    #[other]
    Valid,
    #[eq(u8::MAX)]
    Invalid,
}

let a: Example = Default::default();
let b: Example = 254.into();
let c = a + b;

assert!(a.is_valid());
assert!(b.is_valid());
assert!(c.is_invalid());

let d: Example = c - u8::MAX;

assert!(d.is_nil());
```

## Features

- **HardClamp**: Provides clamped unsigned integers with hard limits.
- **SoftClamp**: Similar to HardClamp but allows invalid states.
- **View**: Combines a data type with a validation type.
- **Guard**: Allows temporary modifications with validation checks before committing changes.
- **Attribute Macro**: `#[clamped(..)]` for creating enums with clamping behavior.

## Contribution

Contributions are welcome! To contribute, please open a PR or a GitHub issue.

## License

This project is dual-licensed under the Apache 2.0 and MIT licenses.

## Contact and Support

For support or further questions, please open a GitHub issue.
