//! Create a builder for TestSys agent configurations
//!
//! Each field of the configuration is given a setter for a templated
//! value, or a typed value.
//!
//! Create a configuration struct and derive `Configuration`.
//! The `crd` attribute must be set as either `Test` or `Resource`
//! A new builder can be created by calling `Config::builder()`
//! To create the test crd from the builder use `build(<NAME>)`
//! ```
//! use configuration_build_derive::Configuration;
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Default, Configuration)]
//! #[crd("Test")]
//! struct Config{
//!     field: Option<String>
//! }
//!
//! let test_crd = Config::builder().image("agent image").build("test name");
//! ```

use crate::derive::{build_struct, impl_configuration};
use proc_macro::{self, TokenStream};
mod derive;

#[macro_use]
extern crate quote;
#[proc_macro_derive(Configuration, attributes(crd))]
pub fn derive_configuration(input: TokenStream) -> TokenStream {
    // Parse the string representation
    let ast = syn::parse(input).unwrap();

    // impl Configuration
    let impl_conf = impl_configuration(&ast);

    // Create the builder struct
    let builder = build_struct(&ast);

    quote! {
        #impl_conf
        #builder
    }
    .into()
}
