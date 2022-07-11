// Copyright (c) 2014 by SiegeLord
//
// All rights reserved. Distributed under LGPL 3.0. For full terms see the file LICENSE.

use crate::lexer::{Error, Source, Span};
use crate::parser::ConfigString;

pub trait Visitor<'l>
{
	fn start_element(&mut self, src: &Source<'l>, name: ConfigString<'l>) -> Result<(), Error>;
	fn end_element(&mut self) -> Result<(), Error>;

	fn set_table(&mut self, src: &Source<'l>, span: Span) -> Result<(), Error>;
	fn set_tagged_array(
		&mut self, src: &Source<'l>, span: Span, tag: ConfigString<'l>,
	) -> Result<(), Error>;
	fn set_tagged_table(
		&mut self, src: &Source<'l>, span: Span, tag: ConfigString<'l>,
	) -> Result<(), Error>;
	fn set_array(&mut self, src: &Source<'l>, span: Span) -> Result<(), Error>;
	fn append_string(&mut self, src: &Source<'l>, string: ConfigString<'l>) -> Result<(), Error>;
	fn expand(&mut self, src: &Source<'l>, name: ConfigString<'l>) -> Result<(), Error>;
}
