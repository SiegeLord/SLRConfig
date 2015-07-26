// Copyright (c) 2014 by SiegeLord
//
// All rights reserved. Distributed under LGPL 3.0. For full terms see the file LICENSE.

/*!
This crate implements the parsing for the SLRConfig format. Basic usage revolves
around the creation and use of the `ConfigElement` type, like so:

~~~
extern crate slr_config;

use slr_config::{ConfigElement};
use std::path::Path;

fn main()
{
	// Parse config element from value.
	let (root, _) = ConfigElement::from_str(Path::new("<dummy>"), "key = value").unwrap();
	assert_eq!(root.as_table().unwrap()["key"].as_value().unwrap(), "value");

	// Create a new table and print it to a string.
	let mut root = ConfigElement::new_table();
	let val = ConfigElement::new_value("value");
	root.insert("key", val);
	assert_eq!(root.to_string(), "key = value\n");
}
~~~
*/


extern crate slr_lexer as lex;

pub use parser::*;
pub use printer::*;
pub use visitor::*;
pub use config_element::*;
pub use from_element::*;

#[macro_use]
mod from_element;
mod parser;
mod visitor;
mod config_element;
mod printer;
mod test;
