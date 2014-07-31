// Copyright (c) 2014 by SiegeLord
//
// All rights reserved. Distributed under LGPL 3.0. For full terms see the file LICENSE.

#![feature(macro_rules)]

extern crate lex = "slr_lexer";

use lex::Lexer;
use lex::Token;
use std::collections::hashmap::HashMap;

pub trait Visitor<'l, E>
{
	fn start_string_entry(&mut self, name: Token<'l>) -> Result<(), E>;
	fn start_table_entry(&mut self, name: Token<'l>) -> Result<(), E>;
	fn start_array_entry(&mut self, name: Token<'l>) -> Result<(), E>;
}

impl<'l> Visitor<'l, ()> for ()
{
	fn start_string_entry(&mut self, name: Token<'l>) -> Result<(), ()>
	{
		println!("Started string: {}", name);
		Ok(())
	}

	fn start_table_entry(&mut self, name: Token<'l>) -> Result<(), ()>
	{
		Ok(())
	}

	fn start_array_entry(&mut self, name: Token<'l>) -> Result<(), ()>
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
				println!("{}", err.text);
				return;
			},
			None => None
		}
	}
}

impl<'l, 'm, E, V: Visitor<'l, E>> Parser<'l, 'm, V>
{
	fn parse_table_contents(&mut self)
	{
		loop
		{
			match get_token!(self.lexer.next())
			{
				Some(cur_tok) =>
				{
					if !cur_tok.kind.is_string()
					{
						fail!("Expected string")
					}
					let next_tok = get_token!(self.lexer.next()).expect("Unexpected EOF");
					if next_tok.kind != lex::Assign
					{
						fail!("Expected '='")
					}
					let third_token = get_token!(self.lexer.next()).expect("Unexpected EOF");
					if third_token.kind.is_string()
					{
						self.visitor.start_string_entry(cur_tok);
					}
					else
					{
						fail!("Expected string");
					}
				}
				None => return
			}
		}
	}
}

pub fn parse_source<'l>(filename: &'l str, source: &'l str)
{
	let mut lexer = Lexer::new(filename, source);
	let mut visitor = ();
	let mut parser = Parser{ lexer: lexer, visitor: &mut visitor };
	parser.parse_table_contents();
}
