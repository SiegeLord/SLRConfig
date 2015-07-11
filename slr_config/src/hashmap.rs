// Copyright (c) 2014 by SiegeLord
//
// All rights reserved. Distributed under LGPL 3.0. For full terms see the file LICENSE.

use std::collections::HashMap;

use visitor::Visitor;
use lex::Error;
use parser::ConfigString;

pub use self::ConfigElement::*;

pub enum ConfigElement
{
	Value(String),
	Table(HashMap<String, ConfigElement>),
	Array(Vec<ConfigElement>),
}

impl ConfigElement
{
	pub fn new_table() -> ConfigElement
	{
		Table(HashMap::new())
	}

	pub fn new_value() -> ConfigElement
	{
		Value(String::new())
	}

	pub fn new_array() -> ConfigElement
	{
		Array(Vec::new())
	}
}

pub struct HashmapVisitor
{
	_root: HashMap<String, ConfigElement>,
	assign_name: String,
	//~ current_element: ConfigElement,
}

impl HashmapVisitor
{
	pub fn new() -> HashmapVisitor
	{
		HashmapVisitor
		{
			_root: HashMap::new(),
			assign_name: "".to_string(),
			//~ current_element: ConfigElement::new_value()
		}
	}
}

impl<'l> Visitor<'l, Error> for HashmapVisitor
{
	fn table_element(&mut self, name: ConfigString<'l>) -> Result<(), Error>
	{
		self.assign_name.clear();
		name.write_string(&mut self.assign_name);
		println!("Table element: {}", self.assign_name);
		Ok(())
	}
	
	fn append_string(&mut self, string: ConfigString<'l>) -> Result<(), Error>
	{
		println!("String appended: {}", string.to_string());
		Ok(())
	}

	fn start_table(&mut self) -> Result<(), Error>
	{
		println!("Started table.");
		Ok(())
	}
	
	fn end_table(&mut self) -> Result<(), Error>
	{
		println!("Ended table.");
		Ok(())
	}

	fn start_array(&mut self) -> Result<(), Error>
	{
		println!("Started array.");
		Ok(())
	}
	
	fn end_array(&mut self) -> Result<(), Error>
	{
		println!("Ended array.");
		Ok(())
	}

	fn array_element(&mut self) -> Result<(), Error>
	{
		println!("Array element");
		Ok(())
	}
}

