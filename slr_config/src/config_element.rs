// Copyright (c) 2014 by SiegeLord
//
// All rights reserved. Distributed under LGPL 3.0. For full terms see the file LICENSE.

use std::collections::BTreeMap;
use std::io;
use std::fmt::{self, Display, Formatter};
use std::mem;
use std::path::Path;
use std::str::from_utf8;

use visitor::Visitor;
use lex::{Error, Span, Source};
use parser::{ConfigString, parse_source};
use printer::Printer;

pub use self::ConfigElementKind::*;

pub struct ConfigElement
{
	pub kind: ConfigElementKind,
	pub span: Span,
}

pub enum ConfigElementKind
{
	Value(String),
	Table(BTreeMap<String, ConfigElement>),
	Array(Vec<ConfigElement>),
}

impl ConfigElement
{
	pub fn new_table() -> ConfigElement
	{
		ConfigElement{ kind: Table(BTreeMap::new()), span: Span::new() }
	}

	pub fn new_value<T: ToString>(value: T) -> ConfigElement
	{
		ConfigElement{ kind: Value(value.to_string()), span: Span::new() }
	}

	pub fn new_array() -> ConfigElement
	{
		ConfigElement{ kind: Array(Vec::new()), span: Span::new() }
	}

	pub fn from_str<'l>(filename: &'l Path, source: &'l str) -> Result<(ConfigElement, Source<'l>), Error>
	{
		let mut root = ConfigElement::new_table();
		let src = try!(root.fill_from_str(filename, source));
		Ok((root, src))
	}

	pub fn fill_from_str<'l>(&mut self, filename: &'l Path, source: &'l str) -> Result<(Source<'l>), Error>
	{
		assert!(self.as_table().is_some());
		let mut root = ConfigElement::new_table();
		mem::swap(&mut root, self);
		let mut visitor = ConfigElementVisitor::new(root);
		parse_source(filename, source, &mut visitor).map(|src|
		{
			mem::swap(&mut visitor.extract_root(), self);
			src
		})
	}

	pub fn as_table(&self) -> Option<&BTreeMap<String, ConfigElement>>
	{
		match self.kind
		{
			Table(ref table) => Some(table),
			_ => None
		}
	}

	pub fn as_table_mut(&mut self) -> Option<&mut BTreeMap<String, ConfigElement>>
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

	pub fn insert<T: ToString>(&mut self, name: T, elem: ConfigElement)
	{
		match self.kind
		{
			Table(ref mut table) =>
			{
				table.insert(name.to_string(), elem);
			},
			Array(ref mut array) =>
			{
				array.push(elem);
			},
			_ => panic!("Trying to insert an element into a value!")
		}
	}

	pub fn print<W: io::Write>(&self, name: Option<&str>, is_root: bool, p: &mut Printer<W>) -> Result<(), io::Error>
	{
		match self.kind
		{
			Value(ref val) => try!(p.value(name, &val)),
			Table(ref table) =>
			{
				try!(p.start_table(name, is_root));
				for (k, v) in table
				{
					try!(v.print(Some(k), false, p));
				}
				try!(p.end_table(is_root));
			}
			Array(ref array) =>
			{
				try!(p.start_array(name));
				for v in array
				{
					try!(v.print(None, false, p));
				}
				try!(p.end_array());
			}
		}
		Ok(())
	}
}

impl Display for ConfigElement
{
	fn fmt(&self, formatter: &mut Formatter) -> Result<(), fmt::Error>
	{
		let mut buf = vec![];
		{
			let mut printer = Printer::new(&mut buf);
			try!(self.print(None, true, &mut printer).map_err(|_| fmt::Error));
		}
		try!(write!(formatter, "{}", try!(from_utf8(&buf).map_err(|_| fmt::Error))));
		Ok(())
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
		if stack_size > 1
		{
			if !value_only || self.stack[stack_size - 1].1.as_value().is_some()
			{
				let (name, elem) = self.stack.pop().unwrap();
				self.stack[stack_size - 2].1.insert(name, elem);
			}
		}
	}
}

impl<'l> Visitor<'l, Error> for ConfigElementVisitor
{
	fn table_element(&mut self, name: ConfigString<'l>) -> Result<(), Error>
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

	fn append_string(&mut self, string: ConfigString<'l>) -> Result<(), Error>
	{
		let stack_size = self.stack.len();
		let elem = &mut self.stack[stack_size - 1].1;
		elem.span.combine(string.span);
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
