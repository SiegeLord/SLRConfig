// Copyright (c) 2014 by SiegeLord
//
// All rights reserved. Distributed under LGPL 3.0. For full terms see the file LICENSE.

use std::collections::HashMap;
use std::path::Path;

use visitor::Visitor;
use lex::{Error, Span, Source};
use parser::{ConfigString, parse_source};

pub use self::ConfigElementKind::*;

pub struct ConfigElement
{
	pub kind: ConfigElementKind,
	pub span: Span,
}

pub enum ConfigElementKind
{
	Value(String),
	Table(HashMap<String, ConfigElement>),
	Array(Vec<ConfigElement>),
}

impl ConfigElement
{
	pub fn new_table() -> ConfigElement
	{
		ConfigElement{ kind: Table(HashMap::new()), span: Span::new() }
	}

	pub fn new_value(value: String) -> ConfigElement
	{
		ConfigElement{ kind: Value(value), span: Span::new() }
	}

	pub fn new_array() -> ConfigElement
	{
		ConfigElement{ kind: Array(Vec::new()), span: Span::new() }
	}

	pub fn from_str<'l>(filename: &'l Path, source: &'l str) -> Result<(ConfigElement, Source<'l>), Error>
	{
		ConfigElement::fill_from_str(ConfigElement::new_table(), filename, source)
	}

	pub fn fill_from_str<'l>(root: ConfigElement, filename: &'l Path, source: &'l str) -> Result<(ConfigElement, Source<'l>), Error>
	{
		assert!(root.as_table().is_some());
		let mut visitor = ConfigElementVisitor::new(root);
		parse_source(filename, source, &mut visitor).map(|src| (visitor.extract_root(), src))
	}

	pub fn as_table(&self) -> Option<&HashMap<String, ConfigElement>>
	{
		match self.kind
		{
			Table(ref table) => Some(table),
			_ => None
		}
	}

	pub fn as_table_mut(&mut self) -> Option<&mut HashMap<String, ConfigElement>>
	{
		match self.kind
		{
			Table(ref mut table) => Some(table),
			_ => None
		}
	}

	pub fn as_value(&self) -> Option<&String>
	{
		match self.kind
		{
			Value(ref value) => Some(value),
			_ => None
		}
	}

	pub fn as_value_mut(&mut self) -> Option<&mut String>
	{
		match self.kind
		{
			Value(ref mut value) => Some(value),
			_ => None
		}
	}

	pub fn as_array(&self) -> Option<&Vec<ConfigElement>>
	{
		match self.kind
		{
			Array(ref array) => Some(array),
			_ => None
		}
	}

	pub fn as_array_mut(&mut self) -> Option<&mut Vec<ConfigElement>>
	{
		match self.kind
		{
			Array(ref mut array) => Some(array),
			_ => None
		}
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

struct ConfigElementVisitor
{
	stack: Vec<(String, ConfigElement)>,
}

impl ConfigElementVisitor
{
	fn new(root: ConfigElement) -> ConfigElementVisitor
	{
		ConfigElementVisitor
		{
			stack: vec![("root".to_string(), root)],
		}
	}

	fn extract_root(mut self) -> ConfigElement
	{
		assert!(self.stack.len() <= 2);
		self.collapse_stack(true);
		assert!(self.stack.len() == 1);
		self.stack.pop().unwrap().1
	}

	fn collapse_stack(&mut self, value_only: bool)
	{
		let stack_size = self.stack.len();
		for e in &self.stack
		{
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

impl<'l> Visitor<'l, Error> for ConfigElementVisitor
{
	fn table_element(&mut self, name: ConfigString<'l>, _span: Span) -> Result<(), Error>
	{
		self.collapse_stack(true);
		self.stack.push((name.to_string(), ConfigElement::new_value("".to_string())));
		Ok(())
	}

	fn array_element(&mut self) -> Result<(), Error>
	{
		self.collapse_stack(true);
		self.stack.push(("".to_string(), ConfigElement::new_value("".to_string())));
		Ok(())
	}

	fn append_string(&mut self, string: ConfigString<'l>, span: Span) -> Result<(), Error>
	{
		let stack_size = self.stack.len();
		let elem = &mut self.stack[stack_size - 1].1;
		elem.span.combine(span);
		string.append_to_string(&mut elem.as_value_mut().as_mut().expect("Trying to append a string to a non-value"));
		Ok(())
	}

	fn start_table(&mut self, span: Span) -> Result<(), Error>
	{
		let stack_size = self.stack.len();
		self.stack[stack_size - 1].1 = ConfigElement::new_table();
		self.stack[stack_size - 1].1.span = span;
		Ok(())
	}

	fn end_table(&mut self, _span: Span) -> Result<(), Error>
	{
		self.collapse_stack(true);
		self.collapse_stack(false);
		Ok(())
	}

	fn start_array(&mut self, span: Span) -> Result<(), Error>
	{
		let stack_size = self.stack.len();
		self.stack[stack_size - 1].1 = ConfigElement::new_array();
		self.stack[stack_size - 1].1.span = span;
		Ok(())
	}

	fn end_array(&mut self, _span: Span) -> Result<(), Error>
	{
		self.collapse_stack(true);
		self.collapse_stack(false);
		Ok(())
	}
}
