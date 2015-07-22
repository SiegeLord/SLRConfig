// Copyright (c) 2014 by SiegeLord
//
// All rights reserved. Distributed under LGPL 3.0. For full terms see the file LICENSE.

use lex::{Span, Source, Error};
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
	fn start_element(&mut self, src: &Source<'l>, name: ConfigString<'l>) -> Result<(), E>;
	fn end_element(&mut self) -> Result<(), E>;

	fn set_table(&mut self, src: &Source<'l>, span: Span) -> Result<(), E>;
	fn set_array(&mut self, src: &Source<'l>, span: Span) -> Result<(), E>;
	fn append_string(&mut self, src: &Source<'l>, string: ConfigString<'l>) -> Result<(), E>;
	fn expand(&mut self, src: &Source<'l>, name: ConfigString<'l>)  -> Result<(), E>;
}
