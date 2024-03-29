// Copyright (c) 2014 by SiegeLord
//
// All rights reserved. Distributed under LGPL 3.0. For full terms see the file LICENSE.

pub use ConfigElementKind::*;

use indexmap::IndexMap;
use slr_parser::{parse_source, ConfigString, Error, ErrorKind, Printer, Source, Span, Visitor};
use std::fmt::{self, Display, Formatter};
use std::io;
use std::mem;
use std::path::Path;
use std::str::{from_utf8, FromStr};

/// A configuration element.
#[derive(Clone, Debug)]
pub struct ConfigElement
{
	kind: ConfigElementKind,
	span: Span,
}

// TODO: It's annoying that we lose the span information from Values and Table keys.
/// The kind of the configuration element.
#[derive(Clone, Debug)]
pub enum ConfigElementKind
{
	/// A simple value, containing a string.
	Value(String),
	/// A table, which is a mapping of strings to configuration elements.
	Table(IndexMap<String, ConfigElement>),
	/// A tagged table, which is a mapping of strings to configuration elements
	/// with a string tag.
	TaggedTable(String, IndexMap<String, ConfigElement>),
	/// An array of configuration elements.
	Array(Vec<ConfigElement>),
	/// An array of configuration elements with a string tag.
	TaggedArray(String, Vec<ConfigElement>),
	/// An array of configuration elements, printed with multiple lines.
	MultiLineArray(Vec<ConfigElement>),
	/// An array of configuration elements with a string tag, printed with multiple lines.
	MultiLineTaggedArray(String, Vec<ConfigElement>),
}

impl ConfigElement
{
	/// Creates a new empty table.
	pub fn new_table() -> ConfigElement
	{
		ConfigElement {
			kind: Table(IndexMap::new()),
			span: Span::new(),
		}
	}

	/// Creates a new empty tagged table.
	pub fn new_tagged_table(tag: String) -> ConfigElement
	{
		ConfigElement {
			kind: TaggedTable(tag, IndexMap::new()),
			span: Span::new(),
		}
	}

	/// Creates a new value.
	pub fn new_value<T: ToString>(value: T) -> ConfigElement
	{
		ConfigElement {
			kind: Value(value.to_string()),
			span: Span::new(),
		}
	}

	/// Creates a new array.
	pub fn new_array() -> ConfigElement
	{
		ConfigElement {
			kind: Array(vec![]),
			span: Span::new(),
		}
	}

	/// Creates a new tagged array.
	pub fn new_tagged_array(tag: String) -> ConfigElement
	{
		ConfigElement {
			kind: TaggedArray(tag, vec![]),
			span: Span::new(),
		}
	}

	/// Creates a new multi-line array.
	pub fn new_multi_line_array() -> ConfigElement
	{
		ConfigElement {
			kind: MultiLineArray(vec![]),
			span: Span::new(),
		}
	}

	/// Creates a new multi-line tagged array.
	pub fn new_multi_line_tagged_array(tag: String) -> ConfigElement
	{
		ConfigElement {
			kind: MultiLineTaggedArray(tag, vec![]),
			span: Span::new(),
		}
	}

	/// Parses a source and returns a table. The source will be reset by this
	/// operation, and must not be used with any spans created from a previous
	/// parsing done with that source.
	pub fn from_source<'l>(source: &mut Source<'l>) -> Result<ConfigElement, Error>
	{
		let mut root = ConfigElement::new_table();
		root.from_source_with_init(source)?;
		Ok(root)
	}

	/// Parses a source and returns a table.
	pub fn from_str(src: &str) -> Result<ConfigElement, Error>
	{
		ConfigElement::from_source(&mut Source::new(&Path::new("<anon>"), src))
	}

	/// Updates the elements in this table with new values parsed from source.
	/// If an error occurs, the contents of this table are undefined. The source
	/// will be reset by this operation, and must not be used with any spans
	/// created from a previous lexing done with that source.
	pub fn from_source_with_init<'l, 'm>(&mut self, source: &'m mut Source<'l>)
		-> Result<(), Error>
	{
		assert!(self.as_table().is_some());
		let mut root = ConfigElement::new_table();
		mem::swap(&mut root, self);
		let mut visitor = ConfigElementVisitor::new(root);
		parse_source(source, &mut visitor).map(|_| {
			mem::swap(&mut visitor.extract_root(), self);
		})
	}

	/// Updates the elements in this table with new values parsed from source.
	/// If an error occurs, the contents of this table are undefined.
	pub fn from_str_with_init(&mut self, src: &str) -> Result<(), Error>
	{
		self.from_source_with_init(&mut Source::new(&Path::new("<anon>"), src))
	}

	/// Returns the kind of this element.
	pub fn kind(&self) -> &ConfigElementKind
	{
		&self.kind
	}

	/// Returns the kind of this element.
	pub fn kind_mut(&mut self) -> &mut ConfigElementKind
	{
		&mut self.kind
	}

	/// Returns the span associated with this element.
	pub fn span(&self) -> Span
	{
		self.span
	}

	/// If this is a table, returns a pointer to its contents.
	pub fn as_table(&self) -> Option<&IndexMap<String, ConfigElement>>
	{
		match self.kind
		{
			Table(ref table) | TaggedTable(_, ref table) => Some(table),
			_ => None,
		}
	}

	/// If this is a table, returns its contents.
	pub fn into_table(self) -> Option<IndexMap<String, ConfigElement>>
	{
		match self.kind
		{
			Table(table) | TaggedTable(_, table) => Some(table),
			_ => None,
		}
	}

	/// If this is a table, returns a pointer to its contents.
	pub fn as_table_mut(&mut self) -> Option<&mut IndexMap<String, ConfigElement>>
	{
		match self.kind
		{
			Table(ref mut table) | TaggedTable(_, ref mut table) => Some(table),
			_ => None,
		}
	}

	/// If this is a value, returns a pointer to its contents.
	pub fn as_value(&self) -> Option<&String>
	{
		match self.kind
		{
			Value(ref value) => Some(value),
			_ => None,
		}
	}

	/// If this is a value, returns its contents.
	pub fn into_value(self) -> Option<String>
	{
		match self.kind
		{
			Value(value) => Some(value),
			_ => None,
		}
	}

	/// If this is a value, returns a pointer to its contents.
	pub fn as_value_mut(&mut self) -> Option<&mut String>
	{
		match self.kind
		{
			Value(ref mut value) => Some(value),
			_ => None,
		}
	}

	/// If this is an array, returns a pointer to its contents.
	pub fn as_array(&self) -> Option<&Vec<ConfigElement>>
	{
		match self.kind
		{
			Array(ref array)
			| TaggedArray(_, ref array)
			| MultiLineArray(ref array)
			| MultiLineTaggedArray(_, ref array) => Some(array),
			_ => None,
		}
	}

	/// If this is an array, returns its contents.
	pub fn into_array(self) -> Option<Vec<ConfigElement>>
	{
		match self.kind
		{
			Array(array)
			| TaggedArray(_, array)
			| MultiLineArray(array)
			| MultiLineTaggedArray(_, array) => Some(array),
			_ => None,
		}
	}

	/// If this is an array, returns a pointer to its contents.
	pub fn as_array_mut(&mut self) -> Option<&mut Vec<ConfigElement>>
	{
		match self.kind
		{
			Array(ref mut array)
			| TaggedArray(_, ref mut array)
			| MultiLineArray(ref mut array)
			| MultiLineTaggedArray(_, ref mut array) => Some(array),
			_ => None,
		}
	}

	pub fn tag(&self) -> Option<&String>
	{
		match self.kind
		{
			TaggedTable(ref tag, _)
			| TaggedArray(ref tag, _)
			| MultiLineTaggedArray(ref tag, _) => Some(tag),
			_ => None,
		}
	}

	pub fn tag_mut(&mut self) -> Option<&mut String>
	{
		match self.kind
		{
			TaggedTable(ref mut tag, _)
			| TaggedArray(ref mut tag, _)
			| MultiLineTaggedArray(ref mut tag, _) => Some(tag),
			_ => None,
		}
	}

	/// Insert an element into a table or an array. Panics if self is a value.
	/// `name` is ignored if self is an array.
	pub fn insert<T: ToString>(&mut self, name: T, elem: ConfigElement)
	{
		match self.kind
		{
			Value(_) => panic!("Trying to insert an element into a value!"),
			Table(ref mut table) | TaggedTable(_, ref mut table) =>
			{
				table.insert(name.to_string(), elem);
			}
			Array(ref mut array)
			| TaggedArray(_, ref mut array)
			| MultiLineArray(ref mut array)
			| MultiLineTaggedArray(_, ref mut array) =>
			{
				array.push(elem);
			}
		}
	}

	/// Outputs the string representation of this element into into a printer.
	pub fn print<W: io::Write>(
		&self, name: Option<&str>, is_root: bool, printer: &mut Printer<W>,
	) -> Result<(), io::Error>
	{
		match self.kind
		{
			Value(ref val) => printer.value(name, &val)?,
			Table(ref table) =>
			{
				printer.start_table(name, is_root, table.is_empty())?;
				for (k, v) in table
				{
					v.print(Some(k), false, printer)?;
				}
				printer.end_table(is_root)?;
			}
			TaggedTable(ref tag, ref table) =>
			{
				printer.start_tagged_table(name, tag, is_root, table.is_empty())?;
				for (k, v) in table
				{
					v.print(Some(k), false, printer)?;
				}
				printer.end_table(is_root)?;
			}
			Array(ref array) =>
			{
				let mut one_line = true;
				for v in array
				{
					match v.kind
					{
						Table(ref table) | TaggedTable(_, ref table) =>
						{
							if !table.is_empty()
							{
								one_line = false;
								break;
							}
						}
						_ => (),
					}
				}
				printer.start_array(name, one_line)?;
				for v in array
				{
					v.print(None, false, printer)?;
				}
				printer.end_array()?;
			}
			TaggedArray(ref tag, ref array) =>
			{
				let mut one_line = true;
				for v in array
				{
					match v.kind
					{
						Table(ref table) | TaggedTable(_, ref table) =>
						{
							if !table.is_empty()
							{
								one_line = false;
								break;
							}
						}
						_ => (),
					}
				}
				printer.start_tagged_array(name, tag, one_line)?;
				for v in array
				{
					v.print(None, false, printer)?;
				}
				printer.end_array()?;
			}
			MultiLineArray(ref array) =>
			{
				printer.start_array(name, false)?;
				for v in array
				{
					v.print(None, false, printer)?;
				}
				printer.end_array()?;
			}
			MultiLineTaggedArray(ref tag, ref array) =>
			{
				printer.start_tagged_array(name, tag, false)?;
				for v in array
				{
					v.print(None, false, printer)?;
				}
				printer.end_array()?;
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
			self.print(None, true, &mut printer)
				.map_err(|_| fmt::Error)?;
		}
		write!(formatter, "{}", from_utf8(&buf).map_err(|_| fmt::Error)?)?;
		Ok(())
	}
}

fn visit_error<'l>(span: Span, source: &Source<'l>, msg: &str) -> Result<(), Error>
{
	Err(Error::from_span(
		span,
		Some(source),
		ErrorKind::ParseFailure,
		msg,
	))
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
		ConfigElementVisitor {
			stack: vec![("root".to_string(), root, true)],
		}
	}

	fn extract_root(mut self) -> ConfigElement
	{
		assert!(self.stack.len() == 1);
		self.stack.pop().unwrap().1
	}
}

impl<'l> Visitor<'l> for ConfigElementVisitor
{
	fn start_element(&mut self, _src: &Source<'l>, name: ConfigString<'l>) -> Result<(), Error>
	{
		self.stack.push((
			name.to_string(),
			ConfigElement::new_value("".to_string()),
			false,
		));
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
				Table(_) =>
				{
					return visit_error(string.span, src, "Cannot append a string to a table")
				}
				TaggedTable(_, _) =>
				{
					return visit_error(
						string.span,
						src,
						"Cannot append a string to a tagged table",
					)
				}
				Array(_) | MultiLineArray(_) =>
				{
					return visit_error(string.span, src, "Cannot append a string to an array")
				}
				TaggedArray(_, _) | MultiLineTaggedArray(_, _) =>
				{
					return visit_error(
						string.span,
						src,
						"Cannot append a string to a tagged array",
					)
				}
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

	fn set_tagged_table(
		&mut self, _src: &Source<'l>, span: Span, tag: ConfigString<'l>,
	) -> Result<(), Error>
	{
		let stack_size = self.stack.len();
		self.stack[stack_size - 1].1 = ConfigElement::new_tagged_table(tag.to_string());
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

	fn set_tagged_array(
		&mut self, _src: &Source<'l>, span: Span, tag: ConfigString<'l>,
	) -> Result<(), Error>
	{
		let stack_size = self.stack.len();
		self.stack[stack_size - 1].1 = ConfigElement::new_tagged_array(tag.to_string());
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
				Table(ref table) | TaggedTable(_, ref table) =>
				{
					found_element = table.get(&name).map(|v| v.clone());
				}
				Array(ref array)
				| TaggedArray(_, ref array)
				| MultiLineArray(ref array)
				| MultiLineTaggedArray(_, ref array) =>
				{
					found_element = <usize>::from_str(&name)
						.ok()
						.and_then(|idx| array.get(idx))
						.map(|v| v.clone());
				}
			}
			if found_element.is_some()
			{
				break;
			}
		}

		if found_element.is_none()
		{
			return visit_error(
				span,
				src,
				&format!("Could not find an element named `{}`", name),
			);
		}
		let found_element = found_element.unwrap();

		let stack_size = self.stack.len();
		let lhs_is_initialized = self.stack[stack_size - 1].2;
		if lhs_is_initialized
		{
			match self.stack[stack_size - 1].1.kind
			{
				Value(ref mut lhs_val) => match found_element.kind
				{
					Value(ref found_val) => lhs_val.push_str(found_val),
					Table(_) => return visit_error(span, src, "Cannot append a table to a value"),
					TaggedTable(_, _) =>
					{
						return visit_error(span, src, "Cannot append a tagged table to a value")
					}
					Array(_) | MultiLineArray(_) =>
					{
						return visit_error(span, src, "Cannot append an array to a value")
					}
					TaggedArray(_, _) | MultiLineTaggedArray(_, _) =>
					{
						return visit_error(span, src, "Cannot append an tagged array to a value")
					}
				},
				Table(_) => return visit_error(span, src, "Cannot append to a table"),
				TaggedTable(_, _) =>
				{
					return visit_error(span, src, "Cannot append to a tagged table")
				}
				Array(_) | MultiLineArray(_) =>
				{
					return visit_error(span, src, "Cannot append to an array")
				}
				TaggedArray(_, _) | MultiLineTaggedArray(_, _) =>
				{
					return visit_error(span, src, "Cannot append to a tagged array")
				}
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
