// Copyright (c) 2014 by SiegeLord
//
// All rights reserved. Distributed under LGPL 3.0. For full terms see the file LICENSE.

/*!
This crate implements the parsing for the SLRConfig format. Basic usage revolves
around the creation and use of the `ConfigElement` type, like so:

~~~
#[macro_use]
extern crate serde_derive;
extern crate slr_config;

use slr_config::{to_element, from_element, ConfigElement};
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

	// You can use Serde as well.
	#[derive(Serialize, Deserialize)]
	struct TestSchema
	{
		key: u32,
		arr: Vec<u32>,
	}

	let mut schema = TestSchema {
		key: 0,
		arr: vec![],
	};
	schema = from_element(&ConfigElement::from_str("key = 5, arr = [1, 2]").unwrap(), None).unwrap();
	assert_eq!(schema.key, 5);
	assert_eq!(schema.arr.len(), 2);
	assert_eq!(schema.arr[0], 1);
	assert_eq!(schema.arr[1], 2);

	let elem = to_element(&schema).unwrap();
	assert_eq!(elem.as_table().unwrap()["key"].as_value().unwrap(), "5");
	assert_eq!(elem.as_table().unwrap()["arr"].as_array().unwrap()[0].as_value().unwrap(), "1");
}
~~~
*/

extern crate slr_parser;
#[macro_use]
extern crate serde;
extern crate indexmap;

pub use config_element::*;
pub use de::from_element;
pub use ser::to_element;
pub use slr_parser::{Error, ErrorKind, Source};

mod config_element;

#[cfg(test)]
#[macro_use]
extern crate serde_derive;

#[cfg(test)]
mod test;

mod de;
mod ser;
