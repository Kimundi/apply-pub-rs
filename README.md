apply-pub-rs
============

A Rust syntax extension for applying the `pub` visibility modifer to many items at once

Right now the attribute applies to every possible child AST element that could have
public visibility, including:

- `use`
- `static`
- `fn`, both standalone and methods/associated ones
- `mod`
- `type`, `struct` and `enum`
- `trait`
- symbols in `extern {}` blocks.

# Example

Add this to your `Cargo.toml`:

```toml
[dependencies.apply-pub-rs]
git = "https://github.com/Kimundi/apply-pub-rs"
```

To load the extension and use it:

```rust
#![feature(phase)]

#[phase(plugin)]
extern crate apply_pub = "apply-pub-rs";

#[apply_pub]
mod foo {
    fn bar() {}
    mod baz {
        fn qux() {}
    }
}

fn main() {
    foo::bar();
    foo::baz::qux();
}
```
