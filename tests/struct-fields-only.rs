#![feature(phase)]

#[phase(plugin)]
extern crate apply_pub;

mod foo {
    #[apply_pub]
    struct Foo {
        a: uint,
        b: uint,
        c: uint,
        d: uint,
        e: uint,
    }
}

#[test]
fn main() {
    let _ = foo::Foo {
        a: 0,
        b: 0,
        c: 0,
        d: 0,
        e: 0,
    };
}
