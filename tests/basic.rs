#[test]
fn basic() {
    #[keywords::r#fn]
    fn foo(a: u32, b: u32, _: keywords! { c: u32, d: u32 }) {
        assert_eq!(a, c);
        assert_eq!(b, d);
    }

    foo(1, 2).c(1).d(2).call();
}

// #[test]
// fn generics() {
//     #[keywords::r#fn]
//     fn foo<T: PartialEq<u32>>(a: u32, b: u32, _: keywords! { c: T, d: u32 }) {
//         assert_eq!(a, c);
//         assert_eq!(b, d);
//     }
// 
//     foo(1, 2).c(1).d(2).call();
// }

// #[test]
// fn impl_block() {
//     struct Foo;
// 
//     #[keywords::block]
//     impl Foo {
//         #[keywords::r#fn]
//         fn foo<T: PartialEq<u32>>(a: u32, b: u32, _: keywords! { c: T, d: u32 }) {
//             assert_eq!(a, c);
//             assert_eq!(b, d);
//         }
//     }
// 
//     let foo = Foo;
// 
//     foo.foo(1, 2).c(1).d(2).call();
// }
