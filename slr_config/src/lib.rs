// Copyright (c) 2014 by SiegeLord
//
// All rights reserved. Distributed under LGPL 3.0. For full terms see the file LICENSE.

/*!
This crate implements the parsing for the SLRConfig format. Basic usage revolves
around the creation and use of the `ConfigElement` type, like so:

~~~
#[macro_use]
extern crate slr_config;

use slr_config::{ConfigElement, ElementRepr};
use std::path::Path;

fn main()
{
	// Parse config element from value.
	let root = ConfigElement::from_str("key = value").unwrap();
	assert_eq!(root.as_table().unwrap()["key"].as_value().unwrap(), "value");

	// Create a new table and print it to a string.
	let mut root = ConfigElement::new_table();
	let val = ConfigElement::new_value("value");
	root.insert("key", val);
	assert_eq!(root.to_string(), "key = value\n");

	// Compile-time schemas automate the above process in many situations.
	slr_def!
	{
		struct TestSchema
		{
			key: u32 = 0,
			arr: Vec<u32> = vec![]
		}
	}

	let mut schema = TestSchema::new();
	schema.from_element(&ConfigElement::from_str("key = 5, arr = [1, 2]").unwrap(), None).unwrap();
	assert_eq!(schema.key, 5);
	assert_eq!(schema.arr.len(), 2);
	assert_eq!(schema.arr[0], 1);
	assert_eq!(schema.arr[1], 2);

	let elem = schema.to_element();
	assert_eq!(elem.as_table().unwrap()["key"].as_value().unwrap(), "5");
	assert_eq!(elem.as_table().unwrap()["arr"].as_array().unwrap()[0].as_value().unwrap(), "1");
}
~~~
*/


extern crate slr_lexer as lex;

pub use parser::*;
pub use printer::*;
pub use visitor::*;
pub use config_element::*;
pub use element_repr::*;
pub use lex::{Error, ErrorKind, Source};

#[macro_use]
mod element_repr;
mod parser;
mod visitor;
mod config_element;
mod printer;
mod test;
