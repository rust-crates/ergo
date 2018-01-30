// -------------------------------------
// ----------- CUSTOM MACROS -----------

#[macro_export]
/// Use to take mutable ownership of _specific_ variables within a closure.
///
/// Closures try to be "smart" about how much scope they capture. If you don't mutate a variable
/// they take `&var`, if you do mutate they take `&mut var`. However, if you require ownership you use
/// `move`, i.e. `move || {... do stuff with var...}`... _right_?
///
/// The problem with `move` is that it moves _every_ variable that is referenced. If you only need
/// to move _one_ variable it can be a pain. Interestingly, you can tell the compiler to only move
/// _specific_ variables like so:
///
/// ```no_compile
/// let x = x;
/// let y = y;
/// // ... etc
/// ```
///
/// But this is quite silly and not obvious to someone who doesn't know about it. Instead, use the
/// `own!` macro and your code will be self documenting.
///
/// > Note: this macro always does `let mut x = x` to mimick its primary usecase of closures
/// > (which infer mutability automatically). If you require non-mutable ownership use `let x = x`
/// > directly.
///
/// # Examples
/// ```
/// #[macro_use] extern crate std_prelude;
///
/// # fn main() {
/// let y = vec![1, 2, 3];
/// let mut x = vec![1, 2, 3];
/// let z = vec![10];
///
/// // create scope in which we mutate `x`
/// {
///     let closure = || {
///         own!(y, z);
///         x.push(4); // mutate reference to x
///         z.push(10);
///
///         println!("&x: {:?}", &x);
///         println!("moved y: {:?}", y);
///         println!("moved z: {:?}", z);
///     };
///
///     closure();
/// }
///
/// println!("&x after: {:?}", x);    // We can still print x!
/// // println!("y after: {:?}", y);  // ERROR: use of moved value
/// // println!("z after: {:?}", z);  // ERROR: use of moved value
/// # }
/// ```
macro_rules! own {
    ( $( $x:ident ),* ) => {
        #[allow(unused_mut)]
        $(
            let mut $x = $x;
        )*
    };
}
