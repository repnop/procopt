extern crate procopt;
extern crate syn;

use procopt::procopt;
use syn::LitStr;

#[procopt]
struct Foo {
    name: LitStr,
    ty: Option<LitStr>,
}

fn main() {}
