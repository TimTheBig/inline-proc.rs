//! This crate provides the ability to write procedural macros directly in your code, instead of
//! having to use another crate.
//!
//! [Repo](https://github.com/SabrinaJewson/inline-proc) - [crates.io](https://crates.io/crates/inline-proc) - [documentation](https://docs.rs/inline-proc)
//!
//! # Example
//! ```
//! use inline_proc::inline_proc;
//!
//! #[inline_proc]
//! mod example {
//!     metadata::ron!(
//!         edition: "2021",
//!         clippy: true,
//!         dependencies: {
//!             "quote": "1",
//!         },
//!         exports: (
//!             bang_macros: {
//!                 "def_func": "define_function",
//!             },
//!         ),
//!     );
//!
//!     pub fn define_function(_: proc_macro::TokenStream) -> proc_macro::TokenStream {
//!         quote::quote!(
//!             fn macro_function() {
//!                 println!("Hello from a proc macro!");
//!             }
//!         ).into()
//!     }
//! }
//!
//! def_func!();
//!
//! macro_function();
//! // => Hello from a proc macro!
//! ```
//!
//! # How It Works
//!
//! `inline-proc` takes your code and puts it in a crate with the path
//! `{temporary directory}/inline-proc-crates/{package name}-{significant package version}-{module name}`.
//! For example, if you call `inline-proc` on Linux from the module `my_module` in `my-nice-crate`
//! which has version `0.7.3`, a temporary crate will be created in
//! `/tmp/inline-proc-crates/my-nice-crate-0.7-my_module`.
//!
//! It then compiles this crate as a `dylib` with Cargo and translates all the outputted errors into
//! errors from the proc macro, so it appears identical to writing the code inline. Note that proc
//! macros cannot currently emit warnings on stable, so you will have to use nightly if you want
//! that.
//!
//! It outputs `macro_rules!` macros that expand to invocations of the private
//! `inline_proc::invoke_inline_macro!` macro. This macro takes in the path of a dylib generated by
//! the `inline_proc` attribute macro, the name of the macro that is inside that dylib, the type of
//! macro that it is (bang/derive/attribute) and the input to the macro. It opens up the dylib and
//! calls the macro, returning its result.
//!
//! # Using the generated macros
//!
//! The macros generated by `#[inline_proc]` can be used directly:
//!
// ! ```
// ! use inline_proc::inline_proc;
// !
// ! #[inline_proc]
// ! mod direct_usage {
// !     metadata::ron!(
// !         edition: "2021",
// !         dependencies: {},
// !         exports: (
// !             bang_macros: { "my_bang_macro": "my_bang_macro" },
// !             derives: { "MyDeriveMacro": "my_derive_macro" },
// !             attributes: { "my_attribute_macro": "my_attribute_macro" },
// !         ),
// !     );
// !     use proc_macro::TokenStream;
// !
// !     pub fn my_bang_macro(_input: TokenStream) -> TokenStream {
// ! #       return TokenStream::new();
// !         todo!()
// !     }
// !     pub fn my_derive_macro(_item: TokenStream) -> TokenStream {
// ! #       return TokenStream::new();
// !         todo!()
// !     }
// !     pub fn my_attribute_macro(_attr: TokenStream, _item: TokenStream) -> TokenStream {
// ! #       return TokenStream::new();
// !         todo!()
// !     }
// ! }
// !
// ! my_bang_macro!(input tokens);
// ! MyDeriveMacro!(struct InnerItem;);
// ! my_attribute_macro!((attribute tokens) struct InnerItem;);
// ! ```
//!
//! This works fine for bang macros, but is not so good for attribute or derive macros. So this
//! crate provides the attribute and derive macros `#[inline_attr]` and `InlineDerive`; they can be
//! used like this:
//!
// ! ```
// ! # use inline_proc::inline_proc;
// ! # #[inline_proc]
// ! # mod indirect_usage {
// ! #     metadata::ron!(
// ! #         edition: "2021",
// ! #         dependencies: {},
// ! #         exports: (
// ! #             derives: { "MyDeriveMacro": "my_derive_macro" },
// ! #             attributes: { "my_attribute_macro": "my_attribute_macro" },
// ! #         ),
// ! #     );
// ! #     use proc_macro::TokenStream;
// ! #
// ! #     pub fn my_derive_macro(_item: TokenStream) -> TokenStream {
// ! #         TokenStream::new()
// ! #     }
// ! #     pub fn my_attribute_macro(_attr: TokenStream, _item: TokenStream) -> TokenStream {
// ! #         TokenStream::new()
// ! #     }
// ! # }
// ! use inline_proc::{InlineDerive, inline_attr};
// !
// ! #[derive(InlineDerive)]
// ! #[inline_derive(MyDeriveMacro)]
// ! struct InnerItem;
// !
// ! #[inline_attr[my_attribute_macro(attribute tokens)]]
// ! struct InnerItem;
// ! ```
//!
//! They expand to the same code as above.
//!
//! # Exporting the macros
//!
//! In order to export your macro, you will first have to change your macro definition to:
//! `"macro_name": ( function: "macro_function", export: true )`. This will do three things:
//!
//! 1. Label the generated `macro_rules!` with `#[macro_export]` and `#[doc(hidden)]`.
//! 1. Have the macro take a path to `inline_proc::invoke_inline_macro`.
//! 1. Suffix the macro's name with `_inner`.
//!
//! You then create a wrapper around it like so:
//!
//! ```
//! # #[inline_proc::inline_proc]
//! # mod macro_export {
//! #     metadata::ron!(
//! #         edition: "2021",
//! #         dependencies: {},
//! #         exports: (
//! #             bang_macros: { "my_macro": ( function: "my_macro", export: true ) },
//! #         ),
//! #     );
//! #     pub fn my_macro(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
//! #         input
//! #     }
//! # }
//! // At the crate root
//!
//! #[doc(hidden)]
//! pub use inline_proc::invoke_inline_macro;
//!
//! // Where your #[inline_proc] is
//!
//! /// This macro does XYZ.
//! #[macro_export]
//! macro_rules! my_macro {
//!     ($($tt:tt)*) => {
//!         $crate::my_macro_inner!($crate::invoke_inline_macro, $($tt)*);
//!     }
//! }
//! ```
//!
//! This level of indirection is necessary as proc macros don't have a way of getting the current
//! crate like MBEs do (`$crate`), so you have to supply it via the MBE method.
//!
//! # Crate attributes
//!
//! Inline procedural macros support inner crate attributes.
//!
// ! ```
// ! use inline_proc::inline_proc;
// !
// ! #[inline_proc]
// ! mod crate_attributes {
// !     metadata::ron!(
// !         edition: "2021",
// !         dependencies: {},
// !         exports: (bang_macros: { "my_bang_macro": "my_bang_macro" }),
// !     );
// !
// !     use proc_macro::TokenStream;
// !
// !     pub fn my_bang_macro(_input: TokenStream) -> TokenStream {
// !        *Box::new(TokenStream::new())
// !     }
// ! }
// ! my_bang_macro!(input tokens);
// ! ```
//!
//! # Caveats
//!
//! This approach comes with several caveats over regular proc macros:
//! - Slower compilation speeds as a second Cargo instance has to be invoked.
//! - Not able to use TOML to define dependencies.
//! - Exporting macros is a pain.
//! - The macros can only be defined in one file.
//! - Errors are a lot less helpful. This is improved a bit by Nightly, but still isn't is good as
//! native proc macro errors.
//! - Derive helper attributes are not supported. The `InlineDerive` macro does reserve the `helper`
//! helper attribute, so you can for example replace `#[my_helper]` with `#[helper[my_helper]]`.

use proc_macro::TokenStream as TokenStream1;
use proc_macro2::{Group, TokenStream};

use proc_macro_error2::{abort, proc_macro_error};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Item, Path, Token};

mod inline_proc;
mod invoke;

/// Write an inline procedural macro.
///
/// This attribute must go on a module, and that module's name must be unique in the entire crate.
///
/// # Metadata
///
/// The module's first item must be an invocation of a `metadata::{format}!` macro, where
/// `{format}` is the chosen format to write the metadata in. Currently we support JSON and
/// [RON](https://github.com/ron-rs/ron), feature-gated with the `json` and `ron` features
/// respectively. In these examples we will use RON because it is shorter and clearer.
///
/// Unfortunately TOML can't be used for this since it is whitespace-sensitive and Rust's lexer
/// strips out all whitespace. Additionally, the `proc_macro_span` feature is unstable so we can't
/// even reconstruct the whitespace.
///
/// ## Metadata Options
///
// / ```
// / use inline_proc::inline_proc;
// /
// / #[inline_proc]
// / mod metadata_options {
// /     metadata::ron!(
// /         // The path to your cargo executable. By default it uses the same one as the one used
// /         // to compile the proc macro (the $CARGO env variable).
// /         cargo: "cargo",
// /
// /         // Whether to pass `--color=always` to Cargo; otherwise the lines printed by Cargo will
// /         // not appear in color. Default is true.
// /         color: true,
// /
// /         // Whether to check the code with Clippy. Default is false.
// /         clippy: true,
// /
// /         // The edition to use. Default is 2015 edition.
// /         edition: "2024",
// /
// /         // The dependencies of the proc macro. This is in the same format as Cargo.toml's
// /         // `[dependencies]` section.
// /         dependencies: {
// /             "proc-macro2": "1",
// /             "syn": ( version: "2", features: ["full"] ),
// /         },
// /
// /         // The path to use for the `inline_proc` crate inside non-exported macros. Defaults to
// /         // `::inline_proc`. Use this if you have renamed the crate.
// /         inline_proc_path: "::inline_proc",
// /
// /         // The macros exported by this module.
// /         exports: (
// /             // The bang macros exported by this module.
// /             bang_macros: {
// /                 // This is a map of the external macro names to paths to the macro functions.
// /                 "my_nice_macro": "my_nice_macro",
// /                 // You can use this form to export the macros. See the crate root for an
// /                 // explanation of how this works.
// /                 "my_public_macro": ( function: "my_nice_macro", export: true ),
// /             },
// /             // The derive macros exported by this module.
// /             derives: {
// /                 "MyDeriveMacro": "my_derive_macro",
// /             },
// /             // The attribute macros exported by this module.
// /             attributes: {
// /                 "my_attribute_macro": "my_attribute_macro",
// /             },
// /         )
// /     );
// /
// /     use proc_macro::TokenStream;
// /
// /     // These functions must have a visibility of pub(super) or higher.
// /
// /     // Bang macros take one token stream and return one token stream.
// /     pub fn my_nice_macro(input: TokenStream) -> TokenStream {
// /         input
// /     }
// /
// /     // Derive macros take one token stream and return one token stream. Unlike other macros
// /     // the original token stream is not destroyed.
// /     pub fn my_derive_macro(_item: TokenStream) -> TokenStream {
// /         TokenStream::new()
// /     }
// /
// /     // Attribute macros take two token streams; the attribute parameters and the item. On a
// /     // regular attribute macro, it looks like this:
// /     //
// /     // #[my_attribute_macro(/* attribute parameters */)]
// /     // struct Whatever; // <-- item
// /     //
// /     // And on an inline attribute macro it looks like this:
// /     //
// /     // #[inline_attr[my_attribute_macro(/* attribute parameters */)]]
// /     // struct Whatever; // <-- item
// /     pub fn my_attribute_macro(_attr: TokenStream, item: TokenStream) -> TokenStream {
// /         item
// /     }
// / }
// / ```
///
/// # Output
///
/// This macro generates a `macro_rules!` macro for each macro listed in `exports`. This macro can
/// be used like so:
///
/// ```ignore
/// // A bang macro
/// my_bang_macro!(input tokens);
/// // A derive macro
/// MyDeriveMacro!(item tokens);
/// // An attribute macro
/// my_attribute_macro!((attribute parameters) item tokens);
/// ```
///
/// However for derive macros and attribute macros it is recommended to use the
/// [`InlineDerive`](derive.InlineDerive.html) and [`#[inline_attr]`](attr.inline_attr.html) macros
/// instead.
#[proc_macro_error]
#[proc_macro_attribute]
pub fn inline_proc(_: TokenStream1, input: TokenStream1) -> TokenStream1 {
    inline_proc::inline_proc(input)
}

#[proc_macro_error]
#[proc_macro]
#[doc(hidden)]
pub fn invoke_inline_macro(input: TokenStream1) -> TokenStream1 {
    invoke::invoke_inline_macro(input)
}

/// Use an inline procedural macro attribute.
///
/// Simply replace where you would usually write `#[my_attr]` or `#[my_attr(params)]` with
/// `#[inline_attr[my_attr]]` and `#[inline_attr[my_attr(params)]]`. Everything else works the
/// same.
///
/// Internally, this macro expands:
///
/// ```ignore
/// #[inline_attr(attr_name(params))]
/// struct Item;
/// ```
/// to:
/// ```ignore
/// attr_name!((params) struct Item);
/// ```
#[proc_macro_attribute]
pub fn inline_attr(params: TokenStream1, item: TokenStream1) -> TokenStream1 {
    let item: TokenStream = item.into();
    let AttrParams { attr_path, tokens } = syn::parse_macro_input!(params);

    quote!(#attr_path!((#tokens) #item);).into()
}

struct AttrParams {
    attr_path: Path,
    tokens: TokenStream,
}
impl Parse for AttrParams {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            attr_path: input.parse()?,
            tokens: input
                .parse::<Option<Group>>()?
                .map_or_else(TokenStream::new, |group| group.stream()),
        })
    }
}

/// Use an inline procedural derive macro.
///
/// Simply replace where you would usually write `#[derive(MyDerive)]` with
/// `#[derive(InlineDerive)] #[inline_derive(MyDerive)]`.
///
/// Since inline procedural derive macros can't define their own helper attributes, this macro
/// reserves the `#[helper]` helper attribute for you to use.
///
/// Internally, this macro expands:
/// ```ignore
/// #[derive(InlineDerive)]
/// #[inline_derive(DeriveName1, DeriveName2)]
/// struct Item;
/// ```
/// to:
/// ```ignore
/// DeriveName1!(struct Item;);
/// DeriveName2!(struct Item;);
/// ```
#[proc_macro_error]
#[proc_macro_derive(InlineDerive, attributes(inline_derive, helper))]
pub fn inline_derive(item: TokenStream1) -> TokenStream1 {
    let mut item: Item = syn::parse_macro_input!(item);

    let attrs = match &mut item {
        Item::Struct(item) => &mut item.attrs,
        Item::Enum(item) => &mut item.attrs,
        Item::Union(item) => &mut item.attrs,
        _ => abort!(item, "Expected struct, enum or union"),
    };

    let attr = match attrs
        .iter()
        .position(|attr| attr.path().is_ident("inline_derive"))
    {
        Some(i) => attrs.remove(i),
        None => abort!(item, "`inline_derive` attribute not present"),
    };
    let derives: Punctuated<Path, Token![,]> =
        match attr.parse_args_with(Punctuated::parse_terminated) {
            Ok(derives) => derives,
            Err(e) => return e.to_compile_error().into(),
        };

    derives
        .iter()
        .map(|derive_path| quote!(#derive_path!(#item);))
        .collect::<TokenStream>()
        .into()
}
