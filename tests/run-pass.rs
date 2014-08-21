#![feature(phase, macro_rules, struct_variant)]

#[phase(plugin)]
extern crate apply_pub = "apply-pub-rs";
extern crate libc;
use libc::c_char;

#[apply_pub]
mod foo {
    extern crate core;
    use self::baz as baz2;
    use self::baz::Foo;
    fn bar() { let _: uint = Foo::foo_fn();  }
    mod baz {
        fn qux() {
            mod foo {
                struct Bar;
                impl Bar {
                    fn bar() {}
                }
            }
            foo::Bar::bar();
        }
        static FOO: uint = 0u;
        static BAR: uint = {
            mod bar {
                static BAR: uint = 100;
            }
            bar::BAR
        };
        struct Data {
            a: uint,
            b: ()
        }
        enum Enum {
            Var1,
            Var2(uint, uint),
            Var3 { a: uint },
        }
        trait Foo {
            fn foo_meth(&self);
            fn foo_fn() -> Self;
        }
        impl Foo for Data {
            fn foo_meth(&self){}
            fn foo_fn() -> Data { Data { a: 0, b: () } }
        }
        type NewData = Data;
        struct Data2;
        struct Data3(uint, uint);
        extern {
            fn toupper(c: ::libc::c_char) -> ::libc::c_char;
        }
    }
}

fn test() {
    use foo::baz::Foo;

    foo::bar();
    foo::baz::qux();
    let _ = foo::baz::FOO;
    let data = foo::baz::Data { a: 0u, b: () };
    data.foo_meth();
    let _: foo::baz::NewData = foo::baz::Foo::foo_fn();

    let _ = foo::baz::Data2;
    let _ = foo::baz::Data3(100u, 200u);

    unsafe {
        assert_eq!(b'A' as c_char, foo::baz::toupper(b'a' as c_char));
    }

    {
        use foo::baz::{Enum, Var1, Var2, Var3};
        let _: Enum = Var1;
        let _ = Var2(0, 1);
        let _ = Var3 { a: 100 };
    }

    impl foo::Foo for uint {
        fn foo_meth(&self) {}
        fn foo_fn() -> uint { 0 }
    }
}

#[apply_pub]
fn a() {
    mod am {
        fn x() {}
    }
    fn b() {
        mod bm {
            fn x() {}
        }
        fn c() {
            mod cm {
                fn x() {}
            }
            fn d() {
                mod dm {
                    fn x() {}
                }
                dm::x()
            }
            cm::x();
            d();
        }
        bm::x();
        c();
    }
    am::x();
    b();
}

macro_rules! items {
    () => {
        fn b(){}
        mod a {}
    }
}

#[apply_pub]
mod test_in_mod {
    items!()
}

// compile_fail:
// macro_rules! rec_items {
//     () => {
//         fn rec_b(){ rec_a(); }
//         fn rec_a(){ rec_b(); }
//     }
// }
// #[apply_pub]
// rec_items!()

#[test]
fn main_test() {
    test();
    a();
    println!("OK")
}
