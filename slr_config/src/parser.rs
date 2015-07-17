// Copyright (c) 2014 by SiegeLord
//
// All rights reserved. Distributed under LGPL 3.0. For full terms see the file LICENSE.

extern crate slr_lexer as lex;

use lex::{Lexer, Token, Error, Span, Source};
use visitor::{Visitor, GetError};
use std::char;
use std::marker::PhantomData;
use std::path::Path;
use std::u32;

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

fn hex_to_char(s: &str) -> char
{
	match u32::from_str_radix(s, 16)
	{
		Ok(n) => char::from_u32(n).unwrap_or('�'),
		Err(_) => '�'
	}
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

	pub fn append_to_string(&self, dest: &mut String)
	{
		match self.kind
		{
			RawString(s) => dest.push_str(s),
			EscapedString(s) =>
			{
				/* Benchmarking has shown this to be faster than computing the exact size. */
				let lb = dest.len() + s.len() - s.chars().filter(|&c| c == '\\').count();
				dest.reserve(lb);
				let mut escape_chars = 0;
				let mut matching_unicode = false;
				let mut unicode_str = "".to_string();

				for mut c in s.chars()
				{
					if escape_chars > 0
					{
						if matching_unicode
						{
							unicode_str.push(c);
						}
						else
						{
							if c == 'u'
							{
								matching_unicode = true;
								escape_chars = 4;
								continue;
							}
							else if c == 'U'
							{
								matching_unicode = true;
								escape_chars = 8;
								continue;
							}
							c = match c
							{
								'n' => '\n',
								'r' => '\r',
								't' => '\t',
								'0' => '\0',
								'\\' => '\\',
								_ => '�'
							};
						}
						escape_chars -= 1;
					}
					else if c == '\\'
					{
						escape_chars = 1;
						continue;
					}
					if escape_chars == 0
					{
						if matching_unicode
						{
							c = hex_to_char(&unicode_str);
							matching_unicode = false;
							unicode_str.clear();
						}
						dest.push(c);
					}
				}
				if matching_unicode
				{
					dest.push(hex_to_char(&unicode_str));
				}
			}
		}
	}

	pub fn to_string(&self) -> String
	{
		let mut dest = String::new();
		self.append_to_string(&mut dest);
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
		try!(self.visitor.start_table(left_brace.span));
		try!(self.parse_table_contents());
		let right_brace = try_eof!(self.lexer.cur_token, Error::from_span(self.lexer.get_source(), left_brace.span, "Unterminated table"));
		try!(self.visitor.end_table(right_brace.span));
		if right_brace.kind != lex::RightBrace
		{
			let error_str = if right_brace.kind == lex::Comma
			{
				"Expected '}' or a string"
			}
			else
			{
				"Expected '}', ',' or a string"
			};
			Error::from_span(self.lexer.get_source(), right_brace.span, error_str)
		}
		else
		{
			self.lexer.next();
			Ok(true)
		}
	}

	fn parse_table_contents(&mut self) -> Result<(), Error>
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
		try!(self.visitor.start_array(left_bracket.span));
		try!(self.parse_array_contents());
		let right_bracket = try_eof!(self.lexer.cur_token, Error::from_span(self.lexer.get_source(), left_bracket.span, "Unterminated array"));
		try!(self.visitor.end_array(right_bracket.span));
		if right_bracket.kind != lex::RightBracket
		{
			let error_str = if right_bracket.kind == lex::Comma
			{
				"Expected ']' or a string"
			}
			else
			{
				"Expected ']', ',' or a string"
			};
			Error::from_span(self.lexer.get_source(), right_bracket.span, error_str)
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

pub fn parse_source<'l, 'm, E: GetError, V: Visitor<'l, E>>(filename: &'l Path, source: &'l str, visitor: &mut V) -> Result<Source<'l>, Error>
{
	let mut lexer = Lexer::new(filename, source);
	lexer.next();
	let mut parser = Parser
	{
		lexer: lexer,
		visitor: visitor,
		error_marker: PhantomData::<E>,
	};
	try!(parser.parse_table_contents());
	match get_token!(parser.lexer.cur_token)
	{
		Some(token) => Error::from_span(parser.lexer.get_source(), token.span, "Expected a string"),
		None => Ok(parser.lexer.get_source().clone())
	}
}
