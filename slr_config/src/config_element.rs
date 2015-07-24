// Copyright (c) 2014 by SiegeLord
//
// All rights reserved. Distributed under LGPL 3.0. For full terms see the file LICENSE.

use std::collections::BTreeMap;
use std::io;
use std::fmt::{self, Display, Formatter};
use std::mem;
use std::path::Path;
use std::str::{FromStr, from_utf8};

use visitor::Visitor;
use lex::{Error, Span, Source};
use parser::{ConfigString, parse_source};
use printer::Printer;

pub use self::ConfigElementKind::*;

/// A configuration element.
#[derive(Clone)]
pub struct ConfigElement
{
	kind: ConfigElementKind,
	span: Span,
}

/// The kind of the configuration element.
#[derive(Clone)]
pub enum ConfigElementKind
{
	/// A simple value, containing a string.
	Value(String),
	/// A table, which is a mapping of strings to configuration elements.
	Table(BTreeMap<String, ConfigElement>),
	/// An array of configuration elements.
	Array(Vec<ConfigElement>),
}

impl ConfigElement
{
	/// Creates a new empty table.
	pub fn new_table() -> ConfigElement
	{
		ConfigElement{ kind: Table(BTreeMap::new()), span: Span::new() }
	}

	/// Creates a new value.
	pub fn new_value<T: ToString>(value: T) -> ConfigElement
	{
		ConfigElement{ kind: Value(value.to_string()), span: Span::new() }
	}

	/// Creates a new array.
	pub fn new_array() -> ConfigElement
	{
		ConfigElement{ kind: Array(Vec::new()), span: Span::new() }
	}

	/// Parses a source and returns a table alongside an annotated source.
	pub fn from_str<'l>(filename: &'l Path, source: &'l str) -> Result<(ConfigElement, Source<'l>), Error>
	{
		let mut root = ConfigElement::new_table();
		let src = try!(root.fill_from_str(filename, source));
		Ok((root, src))
	}

	/// Parses a source and returns a table alongside an annotated source.
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

	pub fn kind(&self) -> &ConfigElementKind
	{
		&self.kind
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
				try!(p.start_table(name, is_root, table.is_empty()));
				for (k, v) in table
				{
					try!(v.print(Some(k), false, p));
				}
				try!(p.end_table(is_root));
			}
			Array(ref array) =>
			{
				let mut one_line = true;
				for v in array
				{
					match v.kind
					{
						Table(ref table) =>
						{
							if !table.is_empty()
							{
								one_line = false;
								break;
							}
						},
						_ => ()
					}
				}
				try!(p.start_array(name, one_line));
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
	// Name, element, initialized
	stack: Vec<(String, ConfigElement, bool)>,
}

impl ConfigElementVisitor
{
	fn new(root: ConfigElement) -> ConfigElementVisitor
	{
		ConfigElementVisitor
		{
			stack: vec![("root".to_string(), root, true)],
		}
	}

	fn extract_root(mut self) -> ConfigElement
	{
		assert!(self.stack.len() == 1);
		self.stack.pop().unwrap().1
	}
}

impl<'l> Visitor<'l, Error> for ConfigElementVisitor
{
	fn start_element(&mut self, _src: &Source<'l>, name: ConfigString<'l>) -> Result<(), Error>
	{
		self.stack.push((name.to_string(), ConfigElement::new_value("".to_string()), false));
		Ok(())
	}

	fn end_element(&mut self) -> Result<(), Error>
	{
		let stack_size = self.stack.len();
		if stack_size > 1
		{
			let (name, elem, _) = self.stack.pop().unwrap();
			self.stack[stack_size - 2].1.insert(name, elem);
		}
		Ok(())
	}

	fn append_string(&mut self, src: &Source<'l>, string: ConfigString<'l>) -> Result<(), Error>
	{
		let stack_size = self.stack.len();
		{
			let elem = &mut self.stack[stack_size - 1].1;
			elem.span.combine(string.span);
			match elem.kind
			{
				Value(ref mut val) => string.append_to_string(val),
				Table(_) => return Error::from_span(src, string.span, "Cannot append a string to a table"),
				Array(_) => return Error::from_span(src, string.span, "Cannot append a string to an array"),
			}
		}
		self.stack[stack_size - 1].2 = true;
		Ok(())
	}

	fn set_table(&mut self, _src: &Source<'l>, span: Span) -> Result<(), Error>
	{
		let stack_size = self.stack.len();
		self.stack[stack_size - 1].1 = ConfigElement::new_table();
		self.stack[stack_size - 1].1.span = span;
		self.stack[stack_size - 1].2 = true;
		Ok(())
	}

	fn set_array(&mut self, _src: &Source<'l>, span: Span) -> Result<(), Error>
	{
		let stack_size = self.stack.len();
		self.stack[stack_size - 1].1 = ConfigElement::new_array();
		self.stack[stack_size - 1].1.span = span;
		self.stack[stack_size - 1].2 = true;
		Ok(())
	}

	fn expand(&mut self, src: &Source<'l>, name: ConfigString<'l>) -> Result<(), Error>
	{
		let mut found_element = None;
		let span = name.span;
		let name = name.to_string();
		// Find the referenced element.
		for &(ref elem_name, ref elem, _) in self.stack.iter().rev()
		{
			// Can't insert currently modified element.
			if *elem_name == name
			{
				continue;
			}
			match elem.kind
			{
				Value(_) => continue,
				Table(ref table) =>
				{
					found_element = table.get(&name).map(|v| v.clone());
				},
				Array(ref array) =>
				{
					found_element = <usize>::from_str(&name).ok().and_then(|idx| array.get(idx)).map(|v| v.clone());
				},
			}
			if found_element.is_some()
			{
				break;
			}
		}

		if found_element.is_none()
		{
			return Error::from_span(src, span, &format!("Could not find an element named `{}`", name));
		}
		let found_element = found_element.unwrap();

		let stack_size = self.stack.len();
		let lhs_is_initialized = self.stack[stack_size - 1].2;
		if lhs_is_initialized
		{
			match self.stack[stack_size - 1].1.kind
			{
				Value(ref mut lhs_val) =>
				{
					match found_element.kind
					{
						Value(ref found_val) => lhs_val.push_str(found_val),
						Table(_) => return Error::from_span(src, span, "Cannot append a table to a value"),
						Array(_) => return Error::from_span(src, span, "Cannot append an array to a value"),
					}
				}
				Table(_) => return Error::from_span(src, span, "Cannot append to a table"),
				Array(_) => return Error::from_span(src, span, "Cannot append to an array"),
			}
		}
		else
		{
			self.stack[stack_size - 1].1 = found_element;
			self.stack[stack_size - 1].2 = true;
		}
		self.stack[stack_size - 1].1.span = span;
		Ok(())
	}
}
