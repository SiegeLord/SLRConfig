// Copyright (c) 2014 by SiegeLord
//
// All rights reserved. Distributed under LGPL 3.0. For full terms see the file LICENSE.

#![feature(macro_rules)]

extern crate lex = "slr_lexer";

use lex::{Lexer, Token, Error};

use std::collections::hashmap::HashMap;

pub trait Visitor<'l, E>
{
	fn start_string_entry(&mut self, name: Token<'l>) -> Result<(), E>;
	fn start_table_entry(&mut self, name: Token<'l>) -> Result<(), E>;
	fn start_array_entry(&mut self, name: Token<'l>) -> Result<(), E>;
}

impl<'l> Visitor<'l, Error> for ()
{
	fn start_string_entry(&mut self, name: Token<'l>) -> Result<(), Error>
	{
		println!("Started string: {}", name);
		Ok(())
	}

	fn start_table_entry(&mut self, name: Token<'l>) -> Result<(), Error>
	{
		Ok(())
	}

	fn start_array_entry(&mut self, name: Token<'l>) -> Result<(), Error>
	{
		Ok(())
	}
}

struct Parser<'l, 'm, V>
{
	lexer: Lexer<'l>,
	visitor: &'m mut V
}

macro_rules! get_token
{
	($tok: expr) =>
	{
		match $tok
		{
			Some(Ok(tok)) =>
			{
				Some(tok)
			},
			Some(Err(ref err)) =>
			{
				return Err(Error::new(err.text.clone()));
			},
			None => None
		}
	}
}

macro_rules! expect_token
{
	($tok: expr, $err: expr) =>
	{
		match get_token!($tok)
		{
			Some(tok) => tok,
			None => return $err
		}
	}
}

impl<'l, 'm, E, V: Visitor<'l, E>> Parser<'l, 'm, V>
{
	fn parse_table_contents(&mut self) -> Result<(), Error>
	{
		loop
		{
			match get_token!(self.lexer.next())
			{
				Some(cur_tok) =>
				{
					if !cur_tok.kind.is_string()
					{
						return Error::from_span(&self.lexer, cur_tok.span, "Expected string");
					}
					let next_tok = expect_token!(self.lexer.next(), Error::from_span(&self.lexer, cur_tok.span, "Unexpected EOF"));
					if next_tok.kind != lex::Assign
					{
						return Error::from_span(&self.lexer, next_tok.span, "Expected '='");
					}
					let third_token = expect_token!(self.lexer.next(), Error::from_span(&self.lexer, next_tok.span, "Unexpected EOF"));
					if third_token.kind.is_string()
					{
						self.visitor.start_string_entry(cur_tok);
					}
					else
					{
						return Error::from_span(&self.lexer, third_token.span, "Expected string");
					}
				},
				None => return Ok(())
			}
		}
		//~ unreachable!();
	}
}

pub fn parse_source<'l>(filename: &'l str, source: &'l str) -> Result<(), Error>
{
	let mut lexer = Lexer::new(filename, source);
	let mut visitor = ();
	let mut parser = Parser{ lexer: lexer, visitor: &mut visitor };
	parser.parse_table_contents()
}
