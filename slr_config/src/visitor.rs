// Copyright (c) 2014 by SiegeLord
//
// All rights reserved. Distributed under LGPL 3.0. For full terms see the file LICENSE.

use lex::{Span, Error};
use parser::ConfigString;

pub trait GetError
{
	fn get_error(&self) -> Error;
}

impl GetError for Error
{
	fn get_error(&self) -> Error
	{
		self.clone()
	}
}

pub trait Visitor<'l, E: GetError>
{
	fn start_table(&mut self, span: Span) -> Result<(), E>;
	fn end_table(&mut self, span: Span) -> Result<(), E>;

	fn start_array(&mut self, span: Span) -> Result<(), E>;
	fn end_array(&mut self, span: Span) -> Result<(), E>;
	
	fn table_element(&mut self, name: ConfigString<'l>) -> Result<(), E>;
	fn array_element(&mut self) -> Result<(), E>;
	fn append_string(&mut self, string: ConfigString<'l>) -> Result<(), E>;
}
