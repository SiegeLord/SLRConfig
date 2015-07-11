// Copyright (c) 2014 by SiegeLord
//
// All rights reserved. Distributed under LGPL 3.0. For full terms see the file LICENSE.

extern crate slr_lexer as lex;
use lex::{Lexer, Token, Error, Span};
use visitor::{Visitor, GetError};
use std::marker::PhantomData;
use std::path::Path;

pub use self::StringKind::*;

#[derive(Clone, Copy, Debug)]
pub struct ConfigString<'l>
{
	pub kind: StringKind<'l>,
	pub span: Span,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StringKind<'l>
{
	EscapedString(&'l str),
	RawString(&'l str),
}

impl<'l> ConfigString<'l>
{
	fn from_token(tok: Token<'l>) -> ConfigString<'l>
	{
		let kind = match tok.kind
		{
			lex::EscapedString(s) => EscapedString(s),
			lex::RawString(s) => RawString(s),
			_ => panic!("Invalid token passed to visitor! {:?}", tok.kind)
		};

		ConfigString{ kind: kind, span: tok.span }
	}

	pub fn write_string(&self, dest: &mut String)
	{
		dest.clear();
		match self.kind
		{
			RawString(s) => dest.push_str(s),
			EscapedString(s) =>
			{
				/* Benchmarking has shown this to be faster than computing the exact size. */
				let lb = s.len() - s.chars().filter(|&c| c == '\\').count();
				dest.reserve(lb);
				let mut escape_next = false;

				for mut c in s.chars()
				{
					if escape_next
					{
						c = match c
						{
							'n' => '\n',
							'r' => '\r',
							't' => '\t',
							'0' => '\0',
							/* TODO: Unicode escapes */
							_ => c
						};
						escape_next = false;
					}
					else if c == '\\'
					{
						escape_next = true;
						continue;
					}
					dest.push(c);
				}
			}
		}
	}

	pub fn to_string(&self) -> String
	{
		let mut dest = String::new();
		self.write_string(&mut dest);
		dest
	}
}

struct Parser<'l, 'm, E, V: 'm>
{
	lexer: Lexer<'l>,
	visitor: &'m mut V,
	error_marker: PhantomData<E>,
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

macro_rules! try_eof
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

impl<'l, 'm, E: GetError, V: Visitor<'l, E>> Parser<'l, 'm, E, V>
{
	fn parse_table(&mut self) -> Result<bool, Error>
	{
		let left_brace = try_eof!(self.lexer.cur_token, Ok(false));
		if left_brace.kind != lex::LeftBrace
		{
			return Ok(false)
		}
		self.lexer.next();
		try!(self.visitor.start_table());
		try!(self.parse_table_contents(false));
		try!(self.visitor.end_table());
		let right_brace = try_eof!(self.lexer.cur_token, Error::from_span(self.lexer.get_source(), left_brace.span, "Unterminated table"));
		if right_brace.kind != lex::RightBrace
		{
			Error::from_span(self.lexer.get_source(), right_brace.span, "Expected '}', ',' or a string")
		}
		else
		{
			self.lexer.next();
			Ok(true)
		}
	}

	fn parse_table_contents(&mut self, is_root: bool) -> Result<(), Error>
	{
		while try!(self.parse_table_element())
		{
			let comma = try_eof!(self.lexer.cur_token, Ok(()));
			if comma.kind == lex::Comma
			{
				self.lexer.next();
			}
		}

		/* Error checking will be done by the calling code */
		Ok(())
	}

	fn parse_table_element(&mut self) -> Result<bool, Error>
	{
		let token = try_eof!(self.lexer.cur_token, Ok(false));
		if token.kind.is_string()
		{
			try!(self.visitor.table_element(ConfigString::from_token(token)));

			let assign = try_eof!(self.lexer.next(), Error::from_span(self.lexer.get_source(), token.span, "Expected '=' or '{' to follow, but got EOF"));
			if assign.kind == lex::Assign
			{
				self.lexer.next();
				if try!(self.parse_array())
				{
					Ok(true)
				}
				else if try!(self.parse_string_expr())
				{
					Ok(true)
				}
				else
				{
					let token = try_eof!(self.lexer.cur_token, Error::from_span(self.lexer.get_source(), assign.span, "Expected '[' or a string to follow, but got EOF"));
					Error::from_span(self.lexer.get_source(), token.span, "Expected '[' or a string")
				}
			}
			else if try!(self.parse_table())
			{
				Ok(true)
			}
			else
			{
				Error::from_span(self.lexer.get_source(), assign.span, "Expected '=' or '{'")
			}
		}
		else
		{
			Ok(false)
		}
	}

	fn parse_array(&mut self) -> Result<bool, Error>
	{
		let left_bracket = try_eof!(self.lexer.cur_token, Ok(false));
		if left_bracket.kind != lex::LeftBracket
		{
			return Ok(false)
		}
		self.lexer.next();
		try!(self.visitor.start_array());
		try!(self.parse_array_contents());
		try!(self.visitor.end_array());
		let right_bracket = try_eof!(self.lexer.cur_token, Error::from_span(self.lexer.get_source(), left_bracket.span, "Unterminated array"));
		if right_bracket.kind != lex::RightBracket
		{
			Error::from_span(self.lexer.get_source(), right_bracket.span, "Expected ']' or ','")
		}
		else
		{
			self.lexer.next();
			Ok(true)
		}
	}

	fn parse_array_contents(&mut self) -> Result<(), Error>
	{
		while try!(self.parse_array_element())
		{
			let comma = try_eof!(self.lexer.cur_token, Ok(()));
			if comma.kind != lex::Comma
			{
				break;
			}
			self.lexer.next();
		}
		/* Error checking will be done by the calling code */
		Ok(())
	}

	fn parse_array_element(&mut self) -> Result<bool, Error>
	{
		let token = try_eof!(self.lexer.cur_token, Ok(false));
		if token.kind.is_string()
		{
			try!(self.visitor.array_element());
			Ok(try!(self.parse_string_expr()))
		}
		else if token.kind == lex::LeftBrace
		{
			try!(self.visitor.array_element());
			Ok(try!(self.parse_table()))
		}
		else if token.kind == lex::LeftBracket
		{
			try!(self.visitor.array_element());
			Ok(try!(self.parse_array()))
		}
		else
		{
			Ok(false)
		}
	}

	fn parse_string_expr(&mut self) -> Result<bool, Error>
	{
		let mut last_span = None;
		loop
		{
			let token = try_eof!(self.lexer.cur_token,
				match last_span
				{
					Some(span) =>
					{
						return Error::from_span(self.lexer.get_source(), span, "Expected a string to follow, but got EOF");
					}
					None =>
					{
						return Ok(false)
					}
				});
			if token.kind.is_string()
			{
				try!(self.visitor.append_string(ConfigString::from_token(token)));
				self.lexer.next();
			}
			else
			{
				match last_span
				{
					Some(span) =>
					{
						return Error::from_span(self.lexer.get_source(), span, "Expected a string to follow, but got EOF");
					}
					None =>
					{
						return Ok(false)
					}
				}
			}

			let tilde = try_eof!(self.lexer.cur_token, Ok(true));
			if tilde.kind != lex::Tilde
			{
				return Ok(true);
			}
			self.lexer.next();
			last_span = Some(tilde.span);
		}
	}
}

pub fn parse_source<'l, 'm, E: GetError, V: Visitor<'l, E>>(filename: &'l Path, source: &'l str, visitor: &mut V) -> Result<(), Error>
{
	let mut lexer = Lexer::new(filename, source);
	lexer.next();
	let mut parser = Parser
	{
		lexer: lexer,
		visitor: visitor,
		error_marker: PhantomData::<E>,
	};
	parser.parse_table_contents(true)
}
