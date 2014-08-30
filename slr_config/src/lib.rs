// Copyright (c) 2014 by SiegeLord
//
// All rights reserved. Distributed under LGPL 3.0. For full terms see the file LICENSE.

#![feature(macro_rules)]

extern crate lex = "slr_lexer";

use lex::{Lexer, Token, Error};

use std::collections::hashmap::HashMap;

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

pub trait Visitor<'l, E>
{
	fn start_assignment(&mut self, path: &[Token<'l>]) -> Result<(), E>;
	fn append_string(&mut self, string: Token<'l>) -> Result<(), E>;
	fn expand(&mut self, path: &[Token<'l>]) -> Result<(), E>;
	fn start_table(&mut self) -> Result<(), E>;
	fn end_table(&mut self) -> Result<(), E>;
}

impl<'l> Visitor<'l, Error> for ()
{
	fn start_assignment(&mut self, path: &[Token<'l>]) -> Result<(), Error>
	{
		println!("Start assignment: {}", path);
		Ok(())
	}

	fn expand(&mut self, path: &[Token<'l>]) -> Result<(), Error>
	{
		println!("Expanded: {}", path);
		Ok(())
	}
	
	fn append_string(&mut self, string: Token<'l>) -> Result<(), Error>
	{
		println!("String appended: {}", string);
		Ok(())
	}

	fn start_table(&mut self) -> Result<(), Error>
	{
		println!("Started table");
		Ok(())
	}
	
	fn end_table(&mut self) -> Result<(), Error>
	{
		println!("Ended table");
		Ok(())
	}
}

struct Parser<'l, 'm, V: 'm>
{
	lexer: Lexer<'l>,
	visitor: &'m mut V,
	path: Vec<Token<'l>>,
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

macro_rules! try
{
	($e: expr) =>
	{
		match $e
		{
			Ok(ok) => ok,
			Err(err) => return Err(err.get_error())
		}
	}
}

impl<'l, 'm, E: GetError, V: Visitor<'l, E>> Parser<'l, 'm, V>
{
	fn parse_table_contents(&mut self, look_for_brace: bool) -> Result<(), Error>
	{
		while try!(self.parse_table_element())
		{
		}
		
		match get_token!(self.lexer.cur_token)
		{
			Some(tok) =>
			{
				if tok.kind == lex::RightBrace && look_for_brace
				{
					Ok(())
				}
				else
				{
					Error::from_span(&self.lexer, tok.span, "Expected assignment or expansion.")
				}
			}
			None => Ok(())
		}
	}

	fn parse_table_element(&mut self) -> Result<bool, Error>
	{
		Ok(try!(self.parse_assignment()))
	}
	
	fn parse_assignment(&mut self) -> Result<bool, Error>
	{
		if !try!(self.parse_index_expr())
		{
			return Ok(false)
		}
		
		try!(self.visitor.start_assignment(self.path.as_slice()));
		
		let assign = expect_token!(self.lexer.cur_token, Error::from_span(&self.lexer, self.path.last().unwrap().span, "Unexpected EOF parsing assignment"));
		if assign.kind != lex::Assign
		{
			return Error::from_span(&self.lexer, assign.span, "Expected '='");
		}
		self.lexer.next();
		
		if !try!(self.parse_expr())
		{
			let cur_token = expect_token!(self.lexer.cur_token, Error::from_span(&self.lexer, assign.span, "Unexpected EOF parsing assignment"));
			return Error::from_span(&self.lexer, cur_token.span, "Expected an expression or '~'");
		}
		Ok(true)
	}

	fn parse_index_expr(&mut self) -> Result<bool, Error>
	{
		let token = expect_token!(self.lexer.cur_token, Ok(false));
		if token.kind.is_string() || token.kind == lex::Root || token.kind == lex::Import
		{
			self.path.clear();
			self.path.push(token);
			self.lexer.next();
			Ok(true)
		}
		else
		{
			Ok(false)
		}
	}
	
	fn parse_expr(&mut self) -> Result<bool, Error>
	{
		Ok(try!(self.parse_no_delete_expr()))
	}
	
	fn parse_no_delete_expr(&mut self) -> Result<bool, Error>
	{
		if try!(self.parse_string_expr())
		{
			Ok(true)
		}
		else
		{
			let brace = expect_token!(self.lexer.cur_token, Ok(false));
			if brace.kind != lex::LeftBrace
			{
				return Ok(false);
			}
			self.lexer.next();
			try!(self.visitor.start_table());
			try!(self.parse_table_contents(true));
			
			let brace = expect_token!(self.lexer.cur_token, Error::from_span(&self.lexer, brace.span, "Unexpected EOF parsing a table"));
			if brace.kind != lex::RightBrace
			{
				return Error::from_span(&self.lexer, brace.span, "Expected '}'");
			}
			self.lexer.next();
			
			try!(self.visitor.end_table());
			Ok(true)
		}
	}
	
	fn parse_string_expr(&mut self) -> Result<bool, Error>
	{
		Ok(try!(self.parse_string_source()))
	}
	
	fn parse_string_source(&mut self) -> Result<bool, Error>
	{
		let token = expect_token!(self.lexer.cur_token, Ok(false));
		if token.kind.is_string()
		{
			try!(self.visitor.append_string(token));
			self.lexer.next();
			Ok(true)
		}
		else if try!(self.parse_expansion())
		{
			try!(self.visitor.expand(self.path.as_slice()));
			Ok(true)
		}
		else
		{
			Ok(false)
		}
	}

	fn parse_expansion(&mut self) -> Result<bool, Error>
	{
		let token = expect_token!(self.lexer.cur_token, Ok(false));
		if token.kind == lex::Dollar
		{
			self.lexer.next();
			Ok(try!(self.parse_index_expr()))
		}
		else
		{
			Ok(false)
		}
	}
}

pub fn parse_source<'l>(filename: &'l str, source: &'l str) -> Result<(), Error>
{
	let mut lexer = Lexer::new(filename, source);
	lexer.next();
	let mut visitor = ();
	let mut parser = Parser{ lexer: lexer, visitor: &mut visitor, path: vec![] };
	parser.parse_table_contents(false)
}
