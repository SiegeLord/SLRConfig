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
		}
	}

	fn write_indent(&mut self) -> Result<(), io::Error>
	{
		for _ in 0..self.depth
		{
			try!(write!(self.writer, "  "));
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
		try!(self.write_indent());
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
		if self.in_array[self.in_array.len() - 1]
		{
			try!(write!(self.writer, ","));
		}
		try!(write!(self.writer, "\n"));
		Ok(())
	}

	pub fn start_array(&mut self, name: Option<&str>) -> Result<(), io::Error>
	{
		match name
		{
			Some(name) =>
			{
				try!(self.write_indent());
				try!(self.write_string(name));
				try!(write!(self.writer, " =\n"));
			}
			_ => ()
		}
		try!(self.write_indent());
		try!(write!(self.writer, "[\n"));
		self.depth += 1;
		self.in_array.push(true);
		Ok(())
	}

	pub fn end_array(&mut self) -> Result<(), io::Error>
	{
		self.depth -= 1;
		self.in_array.pop();
		try!(self.write_indent());
		try!(write!(self.writer, "]"));
		if self.in_array[self.in_array.len() - 1]
		{
			try!(write!(self.writer, ","));
		}
		try!(write!(self.writer, "\n"));
		Ok(())
	}

	pub fn start_table(&mut self, name: Option<&str>, is_root: bool) -> Result<(), io::Error>
	{
		if is_root
		{
			return Ok(())
		}
		match name
		{
			Some(name) =>
			{
				try!(self.write_indent());
				try!(self.write_string(name));
				try!(write!(self.writer, "\n"));
			}
			_ => ()
		}
		try!(self.write_indent());
		try!(write!(self.writer, "{{\n"));
		self.depth += 1;
		self.in_array.push(false);
		Ok(())
	}

	pub fn end_table(&mut self, is_root: bool) -> Result<(), io::Error>
	{
		if is_root
		{
			return Ok(())
		}
		self.depth -= 1;
		self.in_array.pop();
		try!(self.write_indent());
		try!(write!(self.writer, "}}"));
		if self.in_array[self.in_array.len() - 1]
		{
			try!(write!(self.writer, ","));
		}
		try!(write!(self.writer, "\n"));
		Ok(())
	}
}
