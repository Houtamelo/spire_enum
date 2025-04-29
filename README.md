# SpireEnum

A self-proclaimed enum-macro suite for Rust, providing several macros that aim to make enums great again.
*(they never stopped being great, but I needed a punchline)*

- `#[delegated_enum]`: Placed on enums, generates a declarative macro that allows you to delegate impls for your enum in a single line.
  , and/or allows extracting variant types.
- `#[delegated_impl]`: Placed on impl blocks, works in conjunction with `#[delegated_enum]` to generate your enum's delegated impls.
- `#[variant_type_table]`: Place on enums, generates a table type that holds exactly one of each of the enums's variants,
  as well as several useful implementations for that type.
- `#[variant_generic_table]`: Place on enums, works similarly to `#[variant_type_table]`, except each value on the table is
  of a generic parameter instead of the variant's type.
- `#[discriminant_generic_table]`: Place on enums, works similarly to `#[variant_generic_table]`, except this is meant for enums
  with unit variants, accessing the values is used by indexing with the enum variant itself(instead of the variant's type).

- For more info on the table macros, see each macro's documentation.
- For more info on `#[delegated_enum]` and `#[delegated_impl]`, keep reading this file.

## Table of Contents

- [Showcase: What is a delegate enum?](#showcase-what-is-a-delegate-enum)
- [Key Features](#key-features)
- [Overview](#overview)
- [Alternatives](#alternatives)
- [Usage](#usage)
    - [1. `#[delegated_enum]` (Enum attribute macro)](#1-delegated_enum-enum-attribute-macro)
        - [1.1 Basic Usage](#11-basic-usage)
        - [1.2 Conversion Settings](#12-conversion-settings)
            - [1.2.1 `impl_conversions`](#121-impl_conversions)
            - [1.2.2 `impl_enum_try_into_variants`](#122-impl_enum_try_into_variants)
            - [1.2.3 `impl_variants_into_enum`](#123-impl_variants_into_enum)
        - [1.3 Variant Types Generation](#13-variant-types-generation)
            - [1.3.1 `extract_variants`](#131-extract_variants)
            - [1.3.2 `extract_variants( attrs = [attribute_list] )`](#132-extract_variants-attrs--attribute_list-)
            - [1.3.3 `extract_variants( derive(trait_list) )`](#133-extract_variants-derivetrait_list-)
    - [2. `#[delegate_impl]` (Inherent/Trait impl attribute macro)](#2-delegate_impl-inherenttrait-impl-attribute-macro)
        - [2.1 Associated Types, Constants and Static Functions](#21-associated-types-constants-and-static-functions)
    - [3. Variant Attributes](#3-variant-attributes)
        - [3.1 `#[dont_impl_conversions]` / `#[dont_extract]` (Variant attributes)](#31-dont_impl_conversions--dont_extract-variant-attributes)
        - [3.2 `#[delegate_via(|var| var.foo())]` (Variant attribute)](#32-delegate_viavar-varfoo-variant-attribute)
        - [3.3 `#[delegator]` (Variant field attribute)](#33-delegator-variant-field-attribute)
- [Example: Basic Usage](#example-basic-usage)
- [Example: State Machine](#example-state-machine)
- [Troubleshooting](#troubleshooting)
- [Performance](#performance)
- [Contributing](#contributing)

## Showcase: What is a delegate enum?

A "delegate enum" is a common pattern where an enum is created to represent possible types that implement a certain trait.

### Example

You're implementing a state machine for a turret in your video game:

```rust ignore
// Trait that defines what every state should do.
trait IState {
    fn enter(&mut self, turret_body: &mut TurretBody);
    fn exit(self, turret_body: &mut TurretBody);

    /// Returns new state, if changed.
    fn tick(&mut self, turret_body: &mut TurretBody, delta_time: f64) -> Option<State>;
}

// Some state types
#[derive(Serialize, Deserialize)]
struct Idle {
    time_spent: f64,
}

#[derive(Serialize, Deserialize)]
struct Aiming {
    target_id: usize,
}

#[derive(Serialize, Deserialize)]
struct ChargingShot {
    progress: f64,
    shot_type: ShotType,
}

#[derive(Serialize, Deserialize)]
struct CoolingDown {
    time_remaining: f64,
}

// Their impls 
impl IState for Idle { /* ... */ }
impl IState for Aiming { /* ... */ }
impl IState for ChargingShot { /* ... */ }
impl IState for CoolingDown { /* ... */ }

// Now you need some type to store which state a given Turret currently is on.
// The first thing that may come to mind is to simply put it in a `Box<dyn IState>`, but that introduces some problems:
// 
// - Serializing cannot be easily done by just deriving `serde::Serialize`, you need to resort to something like [typetag](https://crates.io/crates/typetag).
// - You lose *some* performance: `dyn` uses dynamic dispatch, and you now need to box your states when storing them.
// 
// In this case, you know all possible variants of a state at compile time, so you could just store those in an enum:
#[derive(Serialize, Deserialize)] // Serialization is very straightforward
enum State {
    Idle(Idle),
    Aiming(Aiming),
    ChargingShot(ChargingShot),
    CoolingDown(CoolingDown),
}

// All good so far, but then, in your main function:

fn main() {
    let mut turrets = some_collection_data_structure();

    while let Some(delta_time) = tick_game_internals() {
        // We want to call tick on the state of every turret.
        for Turret { body, state } in &mut turrets {
            let transition = match state {
                State::Idle(var) => var.tick(body, delta_time),
                State::Aiming(var) => var.tick(body, delta_time),
                State::ChargingShot(var) => var.tick(body, delta_time),
                State::CoolingDown(var) => var.tick(body, delta_time),
            };

            if let Some(new_state) = transition {
                // Exit old state
                let old_state = std::mem::replace(state, new_state);
                match old_state {
                    State::Idle(var) => var.exit(body),
                    State::Aiming(var) => var.exit(body),
                    State::ChargingShot(var) => var.exit(body),
                    State::CoolingDown(var) => var.exit(body),
                }

                // Enter new state
                match state {
                    State::Idle(var) => var.enter(body),
                    State::Aiming(var) => var.enter(body),
                    State::ChargingShot(var) => var.enter(body),
                    State::CoolingDown(var) => var.enter(body),
                }
            }
        }
    }
}


```

You can imagine how verbose this gets, you need to match on all the variants, **every-single-time**, even if you want to call a method that every variant has.

`spire_enum` can help this case in multiple ways.

First, we can generate those big match statements for you, which you replace by calling the generated `delegate_state!` macro:

```rust ignore
use spire_enum_macros::delegated_enum;

#[delegated_enum]
#[derive(Serialize, Deserialize)]
enum State {
    Idle(Idle),
    Aiming(Aiming),
    ChargingShot(ChargingShot),
    CoolingDown(CoolingDown),
}

// Your main function turns into:
fn main() {
    let mut turrets = some_collection_data_structure();

    while let Some(delta_time) = tick_game_internals() {
        // We want to call tick on the state of every turret.
        for Turret { body, state } in &mut turrets {
            // If you want to understand how the generated macro `delegate_state!` works, check the section "1.1 Basic Usage".
            let transition = delegate_state! { state.tick(body, delta_time) };

            if let Some(new_state) = transition {
                // Exit old state
                let old_state = std::mem::replace(state, new_state);
                delegate_state! { old_state.exit(body) }

                // Enter new state
                delegate_state! { state.enter(body) }
            }
        }
    }
}

// The declarative macro `delegate_state!` generates the same code mentioned in the initial example, except it doesn't pollute your view anymore :3

```

`spire_enum` can take it a step further though: instead of having you invoke that macro everytime, why not just let it implement the trait `IState` for your enum?

```rust ignore
use spire_enum_macros::delegate_impl;

#[delegate_impl]
impl IState for State {
    fn enter(&mut self, turret_body: &mut TurretBody);
    fn exit(self, turret_body: &mut TurretBody);

    /// Returns new state, if changed.
    fn tick(&mut self, turret_body: &mut TurretBody, delta_time: f64) -> Option<State>;
}

// Now you can just call those methods directly in the enum:
fn main() {
    let mut turrets = some_collection_data_structure();

    while let Some(delta_time) = tick_game_internals() {
        // We want to call tick on the state of every turret.
        for Turret { body, state } in &mut turrets {
            // If you want to understand how the generated macro `delegate_state!` works, check the section "1.1 Basic Usage".
            let transition = state.tick(body, delta_time);

            if let Some(new_state) = transition {
                // Exit old state
                let old_state = std::mem::replace(state, new_state);
                old_state.exit(body);

                // Enter new state
                state.enter(body);
            }
        }
    }
}

```

But that's not where `spire_enum` stops, in this case, it can also help if you specify the setting `extract_variants` in your enum:

```rust ignore
#[delegated_enum(
    extract_variants(
        attrs = [derive(Serialize, Deserialize)] // applies these attributes to every variant.
    )
)]
#[derive(Serialize, Deserialize)]
enum State {
    Idle { time_spent: f64 },
    Aiming { target_id: usize },
    ChargingShot { progress: f64, shot_type: ShotType },
    CoolingDown { time_remaining: f64 }
}
```

That setting will make the macro also generate and extract the types for each variant, so you don't need to declare the types anymore

This is how your entire file would look like with the usage of `spire_enum`

```rust ignore
use spire_enum_macros::{delegated_enum, delegate_impl};

// Trait
trait IState {
    fn enter(&mut self, turret_body: &mut TurretBody);
    fn exit(self, turret_body: &mut TurretBody);

    /// Returns new state, if changed.
    fn tick(&mut self, turret_body: &mut TurretBody, delta_time: f64) -> Option<State>;
}

// Enum
#[delegated_enum(extract_variants(attrs = [derive(Serialize, Deserialize)]))]
#[derive(Serialize, Deserialize)]
enum State {
    Idle { time_spent: f64 },
    Aiming { target_id: usize },
    ChargingShot { progress: f64, shot_type: ShotType },
    CoolingDown { time_remaining: f64 }
}

// Variant impls
impl IState for Idle { /* ... */ }
impl IState for Aiming { /* ... */ }
impl IState for ChargingShot { /* ... */ }
impl IState for CoolingDown { /* ... */ }

// Enum impl
#[delegate_impl]
impl IState for State {
    fn enter(&mut self, turret_body: &mut TurretBody);
    fn exit(self, turret_body: &mut TurretBody);
    fn tick(&mut self, turret_body: &mut TurretBody, delta_time: f64) -> Option<State>;
}

fn main() {
    let mut turrets = some_collection_data_structure();

    while let Some(delta_time) = tick_game_internals() {
        for Turret { body, state } in &mut turrets {
            let transition = state.tick(body, delta_time);

            if let Some(new_state) = transition {
                let old_state = std::mem::replace(state, new_state);
                old_state.exit(body);
                state.enter(body);
            }
        }
    }
}
```

You thought we were done? Not yet.

When implementing the trait for each variant, there's a good chance you're often converting variants from/into the enum.

```rust ignore
impl IState for Idle {
    /* other functions */

    fn tick(&mut self, turret_body: &mut TurretBody, delta_time: f64) -> Option<State> {
        if let Some(target) = seek_new_target(turret_body) {
            Some(State::Aiming(Aiming { target_id: target.id }))
        } else {
            None
        }
    }
}
```

That's the least of our worries, but it doesn't mean we can't do better.

Let's have `spire_enum` generate conversions implementations too (From<Variant> for Enum, TryFrom<Enum> for Variant),
which can be done with the setting `impl_conversions`:

```rust ignore
#[delegated_enum(
    extract_variants(attrs = [derive(Serialize, Deserialize)]),
    impl_conversions, // <----- This setting
)]
#[derive(Serialize, Deserialize)]
enum State {
    Idle { time_spent: f64 },
    Aiming { target_id: usize },
    ChargingShot { progress: f64, shot_type: ShotType },
    CoolingDown { time_remaining: f64 }
}
```

So now we can just use `.into()`:

```rust ignore
impl IState for Idle {
    /* other functions */

    fn tick(&mut self, turret_body: &mut TurretBody, delta_time: f64) -> Option<State> {
        if let Some(target) = seek_new_target(turret_body) {
            Some(Aiming { target_id: target.id }.into())
        } else {
            None
        }
    }
}
```

There's more this crate can do, but this showcase is already long enough, check the `Usage` section if you want to know more.

## Key Features:

- **Delegation**: Generates inherent/trait implementations for your enums, by delegating to their inner types.
- **Conversions**: Generate `From<>`/`TryFrom<>` implementations between your enums and their variants.
- **Not limited to just plain/simple types**: It properly handles generic parameters, lifetimes (bounds included), where clauses, etc.
- **Hygiene** (IDE friendly!):
    - Proc macros desugar into declarative macros.
    - Token spans are preserved.
    - Macro only uses inputs feed into it, no reflection is performed, no files are read (no IO operations).
    - No state is preserved between macro invocations, each invocation is completely isolated.

## Overview

SpireEnum provides three macros that work together:

1. `#[delegated_enum]` - An attribute macro for defining enums with delegation capabilities.
2. `#[delegate_impl]` - An attribute macro for implementing traits or methods for the enum.
3. (Generated on the fly) `delegate_[enum_name]` - A declarative macro generated by `delegated_enum`, one for each annotated enum.

Macros 1. and 2. work together by:

1. When you apply `#[delegated_enum]` to an enum, the macro parses the enum definition, its settings and variants.
2. Based on the analysis, the macro generates:
    - A declarative macro named `delegate_[enum_name]` that handles the delegation logic.
    - `[Optional]` A new type for each variant.
    - `[Optional]` Conversion (`From<>`, `TryFrom<>`) implementations between the enum and its variants.
3. The `#[delegate_impl]` macro can be applied on trait or inherent implementations of the enum,
   it uses the generated `delegate_[enum_name]` macro to implement the method bodies that need delegation.

## Alternatives

- [enum_delegate](https://crates.io/crates/enum_delegate) - The initial inspiration for this crate,
  I aimed to provide better features while avoiding that crate's drawbacks (the main ones being lack of hygiene, preserving state between macro invocations).
- [delegation](https://crates.io/crates/delegation) - A fork of `enum_delegate`, which solves some of the old one's issues - though
  it takes a different approach compared to `spire_enum`.
- [enum_variant_type](https://crates.io/crates/enum_variant_type) - Provides similar functionality to this crate's `extract_variants` setting.

## Usage

### 1. `#[delegated_enum]` (Enum attribute macro)

This attribute is applied to an enum definition to enable delegation capabilities:

```rust ignore
use spire_enum_macros::delegated_enum;

#[delegated_enum]
pub enum ApiResponse<T> {
    Success(T),
    Error(String),
    Pending { request_id: u64 },
    Timeout,
}
```

The `delegated_enum` attribute supports several optional settings that control how the enum behaves:

#### 1.1 Basic Usage

When used without any settings, `delegated_enum` generates:

- A declarative macro named `delegate_[enum_name]` that can be used to implement delegated traits for the enum.

```rust ignore
#[delegated_enum]
pub enum MediaContent {
    Text(String),
    Image(ImageData),
    Audio { track: AudioFile },
    Video(VideoStream),
    Document(DocumentFile),
}
```

Which generates the declared enum, and this macro:

```rust ignore
macro_rules! delegate_media_content {
    ($_Self:expr => |$arg:ident| $($Rest:tt)*) => {
        match $_Self {
            MediaContent::Text($arg,..) => { $($Rest)* }
            MediaContent::Image($arg,..) => { $($Rest)* }
            MediaContent::Audio{track:$arg,..} => { $($Rest)* }
            MediaContent::Video($arg,..) => { $($Rest)* }
            MediaContent::Document($arg,..) => { $($Rest)* }
        }
    };
    ($_Self:tt $($Rest:tt)*) => {
        match $_Self {
            MediaContent::Text(__var,..) => { __var $($Rest)* }
            MediaContent::Image(__var,..) => { __var $($Rest)* }
            MediaContent::Audio{track,..} => { track$($Rest)* }
            MediaContent::Video(__var,..) => { __var $($Rest)* }
            MediaContent::Document(__var,..) => { __var$($Rest)* }
        }
    };
}

pub(crate) use delegate_media_content;
```

The macro may seem a bit cryptic, but it allows you to manually delegate impls in a very simple way:

```rust ignore
// Let's Imagine that all variants of the enum `MediaContent` implement this trait:
pub trait OnLoad {
    fn on_after_deserialize(&mut self, cfg: &Cfg);
}

// If you were to manually write that implementation for the enum, you would have to:
impl OnLoad for MediaContent {
    fn on_after_deserialize(&mut self, cfg: &Cfg) {
        match self {
            MediaContent::Text(text) => { text.on_after_deserialize(cfg); }
            MediaContent::Image(img) => { img.on_after_deserialize(cfg); }
            MediaContent::Audio { track } => { track.on_after_deserialize(cfg); }
            MediaContent::Video(video) => { video.on_after_deserialize(cfg); }
            MediaContent::Document(doc) => { doc.on_after_deserialize(cfg); }
        }
    }
}

// That can obviously get very tedious if you're frequently using this pattern, 
// that's where the generated macro (`delegate_media_content`) comes in: 
impl OnLoad for MediaContent {
    fn on_after_deserialize(&mut self, cfg: &Cfg) {
        delegate_media_content!(self.on_after_deserialize(cfg));
    }
}
```

Although it can be used manually, the generated declarative macro `delegate_media_content` is used by the `#[delegate_impl]` attribute macro in the
exact same way.

#### 1.2 Conversion Settings

These are specified inside `#[delegated_enum( **here** )]`, separated by commas.

##### 1.2.1 `impl_conversions`

Enable automatic generation of conversion methods between the enum and its variants.
For each `Variant`:

- `TryFrom<Enum>` for `Variant`
- `From<Variant>` for `Enum`

```rust ignore
#[delegated_enum(
    impl_conversions
)]
pub enum MediaContent {
    Text(String),
    Image(ImageData),
    Audio { track: AudioFile },
    Video(VideoStream),
    Document(DocumentFile),
}
```

Which additionally generates:

```rust ignore
impl TryFrom<MediaContent> for String {
    type Error = MediaContent;
    fn try_from(__input: MediaContent) -> Result<Self, Self::Error> {
        if let MediaContent::Text(__var) = __input {
            Ok(__var)
        } else {
            Err(__input)
        }
    }
}

impl From<String> for MediaContent {
    fn from(__input: String) -> Self {
        MediaContent::Text(__input)
    }
}

impl TryFrom<MediaContent> for ImageData {
    type Error = MediaContent;
    fn try_from(__input: MediaContent) -> Result<Self, Self::Error> {
        if let MediaContent::Image(__var) = __input {
            Ok(__var)
        } else {
            Err(__input)
        }
    }
}

impl From<ImageData> for MediaContent {
    fn from(__input: ImageData) -> Self {
        MediaContent::Image(__input)
    }
}

// ... same for each other variant
```

These implementations facilitate conversions between the enums and its possible variants, especially when each variant has a unique type.

This setting is configurable in a per-variant basis, you may skip generating the implementations for certain variants by using the attribute `#
[dont_impl_conversions]`:

```rust ignore
#[delegated_enum(
    impl_conversions
)]
pub enum MediaContent {
    Text(String),
    Image(ImageData),
    Audio { track: AudioFile },
    Video(VideoStream),
    #[dont_impl_conversions] // Will not generate conversions between `DocumentFile` and `MediaContent`
    Document(DocumentFile),
}
```

##### 1.2.2 `impl_enum_try_into_variants`

Similar to `impl_conversions`, except it only generates `TryFrom<Enum>` for each variant.

```rust ignore
#[delegated_enum(
    impl_enum_try_into_variants
)]
pub enum MediaContent {
    Text(String),
    Image(ImageData),
    Audio { track: AudioFile },
    Video(VideoStream),
    Document(DocumentFile),
}
```

##### 1.2.3 `impl_variants_into_enum`

Similar to `impl_conversions`, except it only generates `From<Variant>` for the enum.

```rust ignore
#[delegated_enum(
    impl_variants_into_enum
)]
pub enum MediaContent {
    Text(String),
    Image(ImageData),
    Audio { track: AudioFile },
    Video(VideoStream),
    Document(DocumentFile),
}
```

#### 1.3 Variant Types Generation

These are specified inside `#[delegated_enum( **here** )]`, separated by commas.

##### 1.3.1 `extract_variants`

Generates a new type for each variant:

```rust ignore
#[delegated_enum(extract_variants)]
pub enum SettingsEnum {
    MaxFps(i32),
    DialogueTextSpeed { speed_percent: i32 },
    Vsync(bool),
    Volume(i32),
}
```

Which additionally generates:

```rust ignore
pub struct MaxFps(pub i32);
pub struct Vsync(pub bool);
pub struct Volume(pub i32);

pub struct DialogueTextSpeed {
    pub speed_percent: i32,
}

// And replaces the original enum's variant types with the generated ones:
pub enum SettingsEnum {
    MaxFps(MaxFps),
    Vsync(Vsync),
    Volume(Volume),
    DialogueTextSpeed(DialogueTextSpeed),
}
```

Note that the declarative macro `delegate_[enum_name]` is also generated differently to handle the new variant types.

##### 1.3.2 `extract_variants( attrs = [attribute_list] )`

Applies every attribute in `[attribute_list]` to each generated variant type.

```rust ignore
#[delegated_enum(
    extract_variants(
        attrs = [cfg(test)]
    )
)]
pub enum ApiResource {
    User(UserData),
    Post(PostData),
    Comment(CommentData),
}
```

Which will generate:

```rust ignore
#[cfg(test)]
pub struct User(pub UserData);

#[cfg(test)]
pub struct Post(pub PostData);

#[cfg(test)]
pub struct Comment(pub CommentData);

pub enum ApiResource {
    User(User),
    Post(Post),
    Comment(Comment),
}
```

##### 1.3.3 `extract_variants( derive(trait_list) )`

Shorthand for `attrs = [derive(trait_list)]`

```rust ignore
#[delegated_enum(
    extract_variants(derive(Debug, Clone, Serialize, Deserialize))
)]
pub enum ApiResource {
    User(UserData),
    Post(PostData),
    Comment(CommentData),
}
```

Which will generate:

```rust ignore
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User(pub UserData);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post(pub PostData);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment(pub CommentData);

pub enum ApiResource {
    User(User),
    Post(Post),
    Comment(Comment),
}
```

### 2. `#[delegate_impl]` (Inherent/Trait impl attribute macro)

This attribute should be applied to the enum's implementation blocks:

```rust ignore
use spire_enum_macros::delegate_impl;

#[delegate_impl]
impl<T: Clone> Clone for ApiResponse<T> {
    // The implementation is automatically delegated to the inner types
    // No need to write match statements
    fn clone(&self) -> Self;
}
```

Which generates:

```rust ignore
impl<T: Clone> Clone for ApiResponse<T> {
    fn clone(&self) -> Self {
        delegate_api_response! { self.clone().into() }
    }
}
```

Note that it uses the macro `delegate_api_response`, which would be generated by the enum annotated with `#[delegated_enum]`.

#### 2.1 Associated Types, Constants and Static Functions

Delegating these items is impossible since there's no enum value to match on, you must write that particular
item manually if a trait requires these.

Example:

```rust ignore
use spire_enum_macros::{delegated_enum, delegate_impl};

// Suppose you have this trait:
trait ISetting {
    type Inner;
    const DEFAULT_VALUE: Self::Inner;

    fn apply(&self);
    fn read_from_disk() -> Result<Self>;
}

// And you wish to apply it to the enum:
#[delegated_enum]
enum VolumeSetting {
    Main(MainVolume),
    Music(MusicVolume),
    Sfx(SfxVolume),
}

// The only item that can be delegated is `fn apply(&self)`, other items must be manually written:
#[delegate_impl]
impl ISetting for VolumeSetting {
    // Cannot be delegated, you must provide the type.
    type Inner = f64;

    // Cannot be delegated, you must provide the constant's value.
    const DEFAULT_VALUE: Self::Inner = 50.0;

    // Can be delegated, no need to manually write the implementation.
    fn apply(&self);

    // Cannot be delegated, you must write the implementation.
    fn read_from_disk() -> Result<Self> {
        let file = std::fs::read_to_string("user://settings.cfg");
        // rest of your impl ...
    }
}
```

Note that you may still manually write the implementation of `fn apply(&self)`, which will "override" what would be generated by the macro.

### 3. Variant Attributes

Attributes that can be applied on a per-variant basis.

#### 3.1 `#[dont_impl_conversions]` / `#[dont_extract]` (Variant attributes)

```rust ignore
#[delegated_enum(extract_variants, impl_conversions)]
pub enum Config {
    // Don't implement conversion methods (TryFrom<>, From<>) for this variant.
    // Does nothing if `impl_conversions` isn't present.
    #[dont_impl_conversions]
    Default(DefaultConfig),

    // Don't generate/extract this variant.
    // Does nothing if `extract_variants` isn't present.
    #[dont_extract]
    Custom(CustomConfig),

    Legacy(LegacyConfig),
    Simple(SimpleConfig),
}
```

#### 3.2 `#[delegate_via(|var| var.foo())]` (Variant attribute)

When delegating methods, instead of calling the method directly on the variant, call the delegated method on the result of the closure inside
`delegate_via`.

Example:

```rust ignore
#[delegated_enum(extract_variants, impl_conversions)]
pub enum Config {
    Default(DefaultConfig),
    #[delegate_via(|legacy_config| legacy_config.some_fallback())]
    Legacy(LegacyConfig),
    Simple(SimpleConfig),
}
```

This attribute affects how the macro `delegate_[enum_name]` will be generated, in this, changing it:

```rust ignore
// From
macro_rules! delegate_config {
    ($_Self:expr => |$arg:ident| $($Rest:tt)*) => {
        match $_Self {
            Config::Default($arg) => { $($Rest)* }
            Config::Legacy($arg) => { $($Rest)* }
            Config::Simple($arg) => { $($Rest)*}
        }
    };

    ($_Self:tt $($Rest:tt)*) => {
        match $_Self {
            Config::Default(__var) => { __var $($Rest)* }
            Config::Legacy(__var) => { __var $($Rest)* }
            Config::Simple(__var) => { __var$($Rest)* }
        }
    };
}

// To
macro_rules! delegate_config {
    ($_Self:expr => |$arg:ident| $($Rest:tt)*) => {
        match $_Self {
            Config::Default($arg) => { $($Rest)* }
            Config::Legacy(__var) => {
                let __f = (|legacy_config| legacy_config.some_fallback());
                let $arg = __f(__var);
                $($Rest)*
            }
            Config::Simple($arg) => { $($Rest)*}
        }
    };

    ($_Self:tt $($Rest:tt)*) => {
        match $_Self {
            Config::Default(__var) => { __var $($Rest)* }
            Config::Legacy(__var) => {
                let __f = (|legacy_config| legacy_config.some_fallback());
                let __res = __f(__var);
                __res $($Rest)*
            }
            Config::Simple(__var) => { __var$($Rest)* }
        }
    };
}
```

#### 3.3 `#[delegator]` (Variant field attribute)

Use this to delegate method calls to a field of the variant instead of the variant itself.

Example:

```rust ignore
#[delegated_enum(extract_variants, impl_conversions)]
pub enum Config {
    Default(DefaultConfig),
    Legacy { version: i64, #[delegator] config: LegacyConfig },
    Simple(SimpleConfig),
}
```

This will change the Legacy case in the `delegate_[enum_name]` macro:

```ignore
// From
Config::Legacy { version: $ arg, ..} => { $ ( $ Rest) * }

// To
Config::Legacy { config: $ arg, ..} => { $ ( $ Rest) * }
```

## Example: Basic Usage

```rust ignore
use spire_enum_macros::{delegated_enum, delegate_impl};

#[delegated_enum]
pub enum Value<'a, T>  // Generics are supported.
    where T: Display
{
    Integer(i64),
    Float(f64),
    Text(String),
    Boolean(bool),
    List(&'a Vec<T>),
}

#[delegate_impl]
impl<'c, F> std::fmt::Display for Value<'c, F>
    where T: Display
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
}

let some_variant: Value =..;
println!("{some_variant}");
```

## Example: State Machine

SpireEnum is particularly useful for implementing state machines:

```rust ignore
// Each state already has its own variant type declared outside the macro, no need to use `extract_variants`. 
#[delegated_enum(
    impl_conversions // conversions are nice though
)]
pub enum GameState {
    MainMenu(MenuState),
    Playing(PlayState),
    Paused(PauseState),
    GameOver(GameOverState),
    LevelTransition(TransitionState),
}

// Common interface for all states
trait State {
    fn update(&mut self, delta_time: f32);
    fn handle_input(&mut self, input: UserInput) -> StateTransition;
    fn render(&self, renderer: &mut Renderer);
}

// Implementation delegated to each state.
#[delegate_impl]
impl State for GameState {
    fn update(&mut self, delta_time: f32);
    fn handle_input(&mut self, input: UserInput) -> StateTransition;
    fn render(&self, renderer: &mut Renderer);
}
```

## Troubleshooting

The macros provided by this crate carefully parse the inputs provided to it, I aimed to provide helpful error messages as reasonably as I could.
If you encounter a cryptic error message, please open an issue, I'll do what I can to fix it in a timely manner.

### 1. "Cannot find macro `delegate_[enum_name]`"

You're likely using the macro `#[delegate_impl]` outside of the module that contains your enum.
The macro `delegate_[enum_name]` is generated alongside the enum annotated with `#[delegated_enum]`,
you need to import it on other modules where `#[delegate_impl]` is used:

```rust ignore
use path_to_enum_module::{MyEnum, delegate_my_enum};

#[delegate_impl]
impl Foo for MyEnum {
    fn bar(&self);
}
```

## Performance

The delegation macros generate code that is equivalent to what you would write manually with match statements. There is no runtime overhead compared
to manually written code.

## Contributing

The best way to contribute is by using the crate and providing feedback.
Please open an issue if you'd like to request a feature or report a bug. 