// Copyright (c) 2014 by SiegeLord
//
// All rights reserved. Distributed under LGPL 3.0. For full terms see the file LICENSE.

use lex::{get_string_quote_type, StringQuoteType};
use std::io;

pub struct Printer<'l, W: 'l>
{
	writer: &'l mut W,
	depth: u32,
	in_array: Vec<bool>,
	is_empty: Vec<bool>,
	in_root: bool,
}

impl<'l, W: io::Write> Printer<'l, W>
{
	pub fn new(writer: &'l mut W) -> Printer<'l, W>
	{
		Printer
		{
			writer: writer,
			depth: 0,
			in_array: vec![false],
			is_empty: vec![true],
			in_root: false,
		}
	}

	fn write_indent(&mut self) -> Result<(), io::Error>
	{
		for _ in 0..self.depth
		{
			try!(write!(self.writer, "\t"));
		}
		Ok(())
	}

	fn in_array(&self) -> bool
	{
		self.in_array[self.in_array.len() - 1]
	}

	fn is_empty(&self) -> bool
	{
		self.is_empty[self.is_empty.len() - 1]
	}

	fn set_empty(&mut self, empty: bool)
	{
		let l = self.is_empty.len();
		self.is_empty[l - 1] = empty;
	}

	fn start_value(&mut self) -> Result<(), io::Error>
	{
		if !self.in_array()
		{
			if !(self.depth == 0 && self.in_root && self.is_empty())
			{
				try!(write!(self.writer, "\n"));
			}
			try!(self.write_indent());
		}
		if self.in_array() && !self.is_empty()
		{
			try!(write!(self.writer, ", "));
		}
		Ok(())
	}

	fn write_string(&mut self, s: &str) -> Result<(), io::Error>
	{
		match get_string_quote_type(s)
		{
			StringQuoteType::Naked => try!(write!(self.writer, "{}", s)),
			StringQuoteType::Quoted(num_braces) =>
			{
				for _ in 0..num_braces
				{
					try!(write!(self.writer, "{{"));
				}
				try!(write!(self.writer, r#""{}""#, s));
				for _ in 0..num_braces
				{
					try!(write!(self.writer, "}}"));
				}
			}
		}
		Ok(())
	}
	
	pub fn value(&mut self, name: Option<&str>, value: &str) -> Result<(), io::Error>
	{
		try!(self.start_value());
		match name
		{
			Some(name) =>
			{
				try!(self.write_string(name));
				try!(write!(self.writer, " = "));
			}
			_ => ()
		}
		try!(self.write_string(value));
		self.set_empty(false);
		Ok(())
	}

	pub fn start_array(&mut self, name: Option<&str>) -> Result<(), io::Error>
	{
		try!(self.start_value());
		match name
		{
			Some(name) =>
			{
				try!(self.write_string(name));
				try!(write!(self.writer, " = "));
			}
			_ => ()
		}
		try!(write!(self.writer, "["));
		self.set_empty(false);
		self.depth += 1;
		self.in_array.push(true);
		self.is_empty.push(true);
		Ok(())
	}

	pub fn end_array(&mut self) -> Result<(), io::Error>
	{
		self.depth -= 1;
		self.in_array.pop();
		self.is_empty.pop();
		try!(write!(self.writer, "]"));
		Ok(())
	}

	pub fn start_table(&mut self, name: Option<&str>, is_root: bool) -> Result<(), io::Error>
	{
		if is_root
		{
			self.in_root = true;
			return Ok(())
		}
		try!(self.start_value());
		match name
		{
			Some(name) =>
			{
				try!(self.write_string(name));
				try!(write!(self.writer, "\n"));
			}
			_ => ()
		}
		if !self.in_array()
		{
			try!(self.write_indent());
		}
		try!(write!(self.writer, "{{"));

		self.set_empty(false);
		self.depth += 1;
		self.in_array.push(false);
		self.is_empty.push(true);
		Ok(())
	}

	pub fn end_table(&mut self, is_root: bool) -> Result<(), io::Error>
	{
		if !self.is_empty()
		{
			try!(write!(self.writer, "\n"));
		}
		if is_root
		{
			return Ok(())
		}
		self.depth -= 1;
		if !self.is_empty()
		{
			try!(self.write_indent());
		}
		self.in_array.pop();
		self.is_empty.pop();
		try!(write!(self.writer, "}}"));
		Ok(())
	}
}