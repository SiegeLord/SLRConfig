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
	fn end_assignment(&mut self) -> Result<(), E>;
	fn append_string(&mut self, string: Token<'l>) -> Result<(), E>;
	fn expand(&mut self, path: &[Token<'l>]) -> Result<(), E>;
	fn start_table(&mut self) -> Result<(), E>;
	fn end_table(&mut self) -> Result<(), E>;
	fn delete(&mut self) -> Result<(), E>;
}

impl<'l> Visitor<'l, Error> for ()
{
	fn start_assignment(&mut self, path: &[Token<'l>]) -> Result<(), Error>
	{
		println!("Started assignment: {}", path);
		Ok(())
	}

	fn end_assignment(&mut self) -> Result<(), Error>
	{
		println!("Ended assignment");
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

	fn delete(&mut self) -> Result<(), Error>
	{
		println!("Delete");
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
		// TODO: Expansion
	}
	
	fn parse_assignment(&mut self) -> Result<bool, Error>
	{
		if !try!(self.parse_index_expr())
		{
			return Ok(false)
		}
		
		try!(self.visitor.start_assignment(self.path.as_slice()));
		
		let assign = expect_token!(self.lexer.cur_token,
			Error::from_span(&self.lexer, self.path.last().unwrap().span, "Expected a '=' to follow this string literal, but got EOF"));
		if assign.kind != lex::Assign
		{
			return Error::from_span(&self.lexer, assign.span, "Expected '='");
		}
		self.lexer.next();
		
		if !try!(self.parse_expr())
		{
			let cur_token = expect_token!(self.lexer.cur_token,
				Error::from_span(&self.lexer, assign.span, "Expected a RHS to finish this assignment, but got EOF"));
			return Error::from_span(&self.lexer, cur_token.span, "Expected an expression or 'delete'");
		}
		
		try!(self.visitor.end_assignment());
		
		Ok(true)
	}

	fn parse_index_expr(&mut self) -> Result<bool, Error>
	{
		let token = expect_token!(self.lexer.cur_token, Ok(false));
		if token.kind.is_string() || token.kind == lex::Root || token.kind == lex::Import
		{
			self.path.clear();
			self.path.push(token);
			loop
			{
				let start_token = expect_token!(self.lexer.next(), Ok(true));
				if start_token.kind != lex::LeftBracket
				{
					return Ok(true);
				}
				
				let path_token = expect_token!(self.lexer.next(),
					Error::from_span(&self.lexer, start_token.span, "Expected a string literal to continue this index expression, but got EOF"));
				if !path_token.kind.is_string()
				{
					return Error::from_span(&self.lexer, start_token.span, "Expected a string literal");
				}
				self.path.push(path_token);
				
				let end_token = expect_token!(self.lexer.next(),
					Error::from_span(&self.lexer, start_token.span, "Expected a ']' to finish this index expression, but got EOF"));
				if end_token.kind != lex::RightBracket
				{
					return Error::from_span(&self.lexer, end_token.span, "Expected a ']'");
				}
			}
		}
		else
		{
			Ok(false)
		}
	}
	
	fn parse_expr(&mut self) -> Result<bool, Error>
	{
		if try!(self.parse_no_delete_expr())
		{
			Ok(true)
		}
		else
		{
			let token = expect_token!(self.lexer.cur_token, Ok(false));
			if token.kind == lex::Delete
			{
				try!(self.visitor.delete())
				self.lexer.next();
				Ok(true)
			}
			else
			{
				Ok(false)
			}
		}
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
		// TODO: Array contents
	}
	
	fn parse_string_expr(&mut self) -> Result<bool, Error>
	{
		let mut last_span = None;
		loop
		{
			if !try!(self.parse_string_source())
			{
				match last_span
				{
					Some(span) =>
					{
						return Error::from_span(&self.lexer, span, "Expected a string source to finish this concatenation, but got EOF");
					}
					None =>
					{
						return Ok(false)
					}
				}
			}
			
			let expand = expect_token!(self.lexer.cur_token, Ok(true));
			if expand.kind != lex::Tilde
			{
				return Ok(true);
			}
			self.lexer.next();
			last_span = Some(expand.span);
		}
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
