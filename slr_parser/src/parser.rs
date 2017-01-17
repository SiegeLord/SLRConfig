// Copyright (c) 2014 by SiegeLord
//
// All rights reserved. Distributed under LGPL 3.0. For full terms see the file LICENSE.

use lexer::{Error, ErrorKind, Lexer, Source, Span, Token, TokenKind};
use std::char;
use std::marker::PhantomData;
use std::u32;
use visitor::{GetError, Visitor};

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
		Err(_) => '�',
	}
}

impl<'l> ConfigString<'l>
{
	fn new() -> ConfigString<'l>
	{
		ConfigString {
			kind: StringKind::RawString(""),
			span: Span::new(),
		}
	}

	fn from_token(tok: Token<'l>) -> ConfigString<'l>
	{
		let kind = match tok.kind
		{
			TokenKind::EscapedString(s) => StringKind::EscapedString(s),
			TokenKind::RawString(s) => StringKind::RawString(s),
			_ => panic!("Invalid token passed to visitor! {:?}", tok.kind),
		};

		ConfigString { kind: kind, span: tok.span }
	}

	pub fn append_to_string(&self, dest: &mut String)
	{
		match self.kind
		{
			StringKind::RawString(s) => dest.push_str(s),
			StringKind::EscapedString(s) =>
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
								_ => '�',
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

struct Parser<'l, 's, 'm, E, V: 'm>
	where 's: 'l
{
	lexer: Lexer<'l, 's>,
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
				return Err(Error::new(err.kind, err.text.clone()));
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

impl<'l, 's, 'm, E: GetError, V: Visitor<'l, E>> Parser<'l, 's, 'm, E, V>
{
	fn parse_error<T>(&self, span: Span, msg: &str) -> Result<T, Error>
	{
		Err(Error::from_span::<T>(span, Some(self.lexer.get_source()), ErrorKind::ParseFailure, msg))
	}

	fn parse_table(&mut self) -> Result<bool, Error>
	{
		let left_brace = try_eof!(self.lexer.cur_token, Ok(false));
		if left_brace.kind != TokenKind::LeftBrace
		{
			return Ok(false);
		}
		self.lexer.next();
		try!(self.visitor.set_table(self.lexer.get_source(), left_brace.span));
		try!(self.parse_table_contents());
		let right_brace = try_eof!(self.lexer.cur_token, self.parse_error(left_brace.span, "Unterminated table"));
		if right_brace.kind != TokenKind::RightBrace
		{
			let error_str = if right_brace.kind == TokenKind::Comma
			{
				"Expected '}' or a string"
			}
			else
			{
				"Expected '}', ',' or a string"
			};
			self.parse_error(right_brace.span, error_str)
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
			if comma.kind == TokenKind::Comma
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
		let ret = if token.kind.is_string()
		{
			try!(self.visitor.start_element(self.lexer.get_source(), ConfigString::from_token(token)));

			let assign = try_eof!(self.lexer.next(), self.parse_error(token.span, "Expected '=' or '{' to follow, but got EOF"));
			if assign.kind == TokenKind::Assign
			{
				self.lexer.next();
				if try!(self.parse_array())
				{
					true
				}
				else if try!(self.parse_string_expr())
				{
					true
				}
				else
				{
					let token = try_eof!(self.lexer.cur_token,
					                     self.parse_error(assign.span, "Expected '[' or a string to follow, but got EOF"));
					return self.parse_error(token.span, "Expected '[' or a string");
				}
			}
			else if try!(self.parse_table())
			{
				true
			}
			else
			{
				return self.parse_error(assign.span, "Expected '=' or '{'");
			}
		}
		else
		{
			false
		};
		if ret
		{
			try!(self.visitor.end_element());
		}
		Ok(ret)
	}

	fn parse_array(&mut self) -> Result<bool, Error>
	{
		let left_bracket = try_eof!(self.lexer.cur_token, Ok(false));
		if left_bracket.kind != TokenKind::LeftBracket
		{
			return Ok(false);
		}
		self.lexer.next();
		try!(self.visitor.set_array(self.lexer.get_source(), left_bracket.span));
		try!(self.parse_array_contents());
		let right_bracket = try_eof!(self.lexer.cur_token, self.parse_error(left_bracket.span, "Unterminated array"));
		if right_bracket.kind != TokenKind::RightBracket
		{
			let error_str = if right_bracket.kind == TokenKind::Comma
			{
				"Expected ']' or a string"
			}
			else
			{
				"Expected ']', ',' or a string"
			};
			self.parse_error(right_bracket.span, error_str)
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
			if comma.kind != TokenKind::Comma
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
		let ret = if token.kind.is_string() || token.kind == TokenKind::Dollar
		{
			try!(self.visitor.start_element(self.lexer.get_source(), ConfigString::new()));
			try!(self.parse_string_expr())
		}
		else if token.kind == TokenKind::LeftBrace
		{
			try!(self.visitor.start_element(self.lexer.get_source(), ConfigString::new()));
			try!(self.parse_table())
		}
		else if token.kind == TokenKind::LeftBracket
		{
			try!(self.visitor.start_element(self.lexer.get_source(), ConfigString::new()));
			try!(self.parse_array())
		}
		else
		{
			false
		};
		if ret
		{
			try!(self.visitor.end_element());
		}
		Ok(ret)
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
					                     return self.parse_error(span, "Expected a string or '$' to follow, but got EOF");
					                    }
				                     None => return Ok(false),
			                     });
			if token.kind.is_string()
			{
				try!(self.visitor.append_string(self.lexer.get_source(), ConfigString::from_token(token)));
				self.lexer.next();
			}
			else if token.kind == TokenKind::Dollar
			{
				let string_token = try_eof!(self.lexer.next(), self.parse_error(token.span, "Expected a string to follow, but got EOF"));
				if string_token.kind.is_string()
				{
					try!(self.visitor.expand(self.lexer.get_source(), ConfigString::from_token(string_token)));
					self.lexer.next();
				}
				else
				{
					return self.parse_error(string_token.span, "Expected a string");
				}
			}
			else
			{
				match last_span
				{
					Some(span) =>
					{
						return self.parse_error(span, "Expected a string or '$' to follow, but got EOF");
					}
					None => return Ok(false),
				}
			}

			let tilde = try_eof!(self.lexer.cur_token, Ok(true));
			if tilde.kind != TokenKind::Tilde
			{
				return Ok(true);
			}
			self.lexer.next();
			last_span = Some(tilde.span);
		}
	}
}

pub fn parse_source<'l, 'm, E: GetError, V: Visitor<'m, E>>(source: &'m mut Source<'l>, visitor: &mut V) -> Result<(), Error>
{
	let mut lexer = Lexer::new(source);
	lexer.next();
	let mut parser = Parser {
		lexer: lexer,
		visitor: visitor,
		error_marker: PhantomData::<E>,
	};
	try!(parser.parse_table_contents());
	match get_token!(parser.lexer.cur_token)
	{
		Some(token) => parser.parse_error(token.span, "Expected a string"),
		None => Ok(()),
	}
}
