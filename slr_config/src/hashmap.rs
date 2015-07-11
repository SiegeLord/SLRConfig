// Copyright (c) 2014 by SiegeLord
//
// All rights reserved. Distributed under LGPL 3.0. For full terms see the file LICENSE.

use std::collections::HashMap;

use visitor::Visitor;
use lex::Error;
use parser::ConfigString;

pub use self::ConfigElementKind::*;

pub struct ConfigElement
{
	pub kind: ConfigElementKind
}

pub enum ConfigElementKind
{
	Value(String),
	Table(HashMap<String, ConfigElement>),
	Array(Vec<ConfigElement>),
}

impl ConfigElement
{
	fn as_table_mut(&mut self) -> Option<&mut HashMap<String, ConfigElement>>
	{
		match self.kind
		{
			Table(ref mut table) => Some(table),
			_ => None
		}
	}

	fn as_value(&self) -> Option<&String>
	{
		match self.kind
		{
			Value(ref value) => Some(value),
			_ => None
		}
	}

	fn as_value_mut(&mut self) -> Option<&mut String>
	{
		match self.kind
		{
			Value(ref mut value) => Some(value),
			_ => None
		}
	}

	fn as_array_mut(&mut self) -> Option<&mut Vec<ConfigElement>>
	{
		match self.kind
		{
			Array(ref mut array) => Some(array),
			_ => None
		}
	}
}

impl ConfigElement
{
	pub fn new_table() -> ConfigElement
	{
		ConfigElement{ kind: Table(HashMap::new()) }
	}

	pub fn new_value() -> ConfigElement
	{
		ConfigElement{ kind: Value("".to_string()) }
	}

	pub fn new_array() -> ConfigElement
	{
		ConfigElement{ kind: Array(Vec::new()) }
	}

	pub fn insert_element(&mut self, name: String, elem: ConfigElement)
	{
		match self.kind
		{
			Table(ref mut table) =>
			{
				table.insert(name, elem);
			},
			Array(ref mut array) =>
			{
				array.push(elem);
			},
			_ => panic!("Trying to insert an element into a value!")
		}
	}
}

pub struct HashmapVisitor
{
	stack: Vec<(String, ConfigElement)>,
}

impl HashmapVisitor
{
	pub fn new() -> HashmapVisitor
	{
		HashmapVisitor
		{
			stack: vec![("root".to_string(), ConfigElement::new_table())],
		}
	}

	pub fn extract_root(mut self) -> ConfigElement
	{
		assert!(self.stack.len() <= 2);
		self.collapse_stack(true);
		self.stack.pop().unwrap().1
	}

	pub fn collapse_stack(&mut self, value_only: bool)
	{
		let stack_size = self.stack.len();
		println!("Dumping stack");
		for e in &self.stack
		{
			print!("  {}", e.0);
			match e.1.kind
			{
				Table(ref table) => println!(" table {}", table.len()),
				Value(ref val) => println!(" value {}", val),
				Array(ref array) => println!(" array {}", array.len()),
			}
		}
		if stack_size > 1
		{
			if !value_only || self.stack[stack_size - 1].1.as_value().is_some()
			{
				let (name, elem) = self.stack.pop().unwrap();
				self.stack[stack_size - 2].1.insert_element(name, elem);
			}
		}
	}
}

impl<'l> Visitor<'l, Error> for HashmapVisitor
{
	fn table_element(&mut self, name: ConfigString<'l>) -> Result<(), Error>
	{
		println!("Table element: {}", name.to_string());
		self.collapse_stack(true);
		self.stack.push((name.to_string(), ConfigElement::new_value()));
		Ok(())
	}

	fn array_element(&mut self) -> Result<(), Error>
	{
		println!("Array element");
		self.collapse_stack(true);
		self.stack.push(("".to_string(), ConfigElement::new_value()));
		Ok(())
	}

	fn append_string(&mut self, string: ConfigString<'l>) -> Result<(), Error>
	{
		println!("String appended: {}", string.to_string());
		let stack_size = self.stack.len();
		string.append_to_string(&mut self.stack[stack_size - 1].1.as_value_mut().as_mut().expect("Trying to append a string to a non-value"));
		Ok(())
	}

	fn start_table(&mut self) -> Result<(), Error>
	{
		println!("Started table.");
		let stack_size = self.stack.len();
		self.stack[stack_size - 1].1 = ConfigElement::new_table();
		Ok(())
	}

	fn end_table(&mut self) -> Result<(), Error>
	{
		println!("Ended table.");
		self.collapse_stack(true);
		self.collapse_stack(false);
		Ok(())
	}

	fn start_array(&mut self) -> Result<(), Error>
	{
		println!("Started array.");
		let stack_size = self.stack.len();
		self.stack[stack_size - 1].1 = ConfigElement::new_array();
		Ok(())
	}

	fn end_array(&mut self) -> Result<(), Error>
	{
		println!("Ended array.");
		self.collapse_stack(true);
		self.collapse_stack(false);
		Ok(())
	}
}

