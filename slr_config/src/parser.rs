// Copyright (c) 2014 by SiegeLord
//
// All rights reserved. Distributed under LGPL 3.0. For full terms see the file LICENSE.

/* FIXME: Some bug with globs */
extern crate lex = "slr_lexer";

use lex::{Lexer, Token, Error, Span};
use visitor::{Visitor, GetError};

#[deriving(Clone, Show)]
pub struct ConfigString<'l>
{
	pub kind: StringKind<'l>,
	pub span: Span,
}

#[deriving(Clone, Show, PartialEq)]
pub enum StringKind<'l>
{
	EscapedString(&'l str),
	RawString(&'l str),
}

#[deriving(Clone, Show, PartialEq)]
pub enum PathKind
{
	Absolute,
	Relative,
	Import,
}

impl<'l> ConfigString<'l>
{
	fn from_token(tok: Token<'l>) -> ConfigString<'l>
	{
		let kind = match tok.kind
		{
			lex::EscapedString(s) => EscapedString(s),
			lex::RawString(s) => RawString(s),
			_ => fail!("Invalid token passed to visitor! {}", tok.kind)
		};

		ConfigString{ kind: kind, span: tok.span }
	}

	pub fn into_string(&self, dest: &mut String)
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
					dest.push_char(c);
				}
			}
		}
	}
	
	pub fn to_string(&self) -> String
	{
		let mut dest = String::new();
		self.into_string(&mut dest);
		dest
	}
}

struct Parser<'l, 'm, V: 'm>
{
	lexer: Lexer<'l>,
	visitor: &'m mut V,
	path_kind: PathKind,
	path: Vec<ConfigString<'l>>,
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

impl<'l, 'm, E: GetError, V: Visitor<'l, E>> Parser<'l, 'm, V>
{
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

		if !is_root
		{
			/* Error checking will be done by the calling code */
			Ok(())
		}
		else
		{
			match get_token!(self.lexer.cur_token)
			{
				Some(tok) =>
				{
					Error::from_span(self.lexer.get_source(), tok.span, "Expected assignment or expansion")
				},
				None => Ok(())
			}
		}
	}

	fn parse_array_contents(&mut self) -> Result<(), Error>
	{
		try!(self.visitor.array_element());
		while try!(self.parse_array_element())
		{
			let comma = try_eof!(self.lexer.cur_token, Ok(()));
			if comma.kind != lex::Comma
			{
				break;
			}
			self.lexer.next();
			try!(self.visitor.array_element());
		}
		/* Error checking will be done by the calling code */
		Ok(())
	}

	fn parse_table_element(&mut self) -> Result<bool, Error>
	{
		if try!(self.parse_assignment())
		{
			Ok(true)
		}
		else if try!(self.parse_expansion())
		{
			try!(self.visitor.insert_path(self.path_kind, self.path.as_slice()));
			Ok(true)
		}
		else
		{
			Ok(false)
		}
	}

	fn parse_array_element(&mut self) -> Result<bool, Error>
	{
		let cur_token = try_eof!(self.lexer.cur_token, Ok(false));
		let next_token = try_eof!(self.lexer.next_token, Ok(false));

		/* Could be an index, or an expression */
		if cur_token.kind.is_string()
		{
			if next_token.kind == lex::Comma || next_token.kind == lex::RightBracket
			{
				Ok(try!(self.parse_no_delete_expr()))
			}
			else
			{
				Ok(try!(self.parse_assignment()))
			}
		}
		else
		{
			/* Try the expression and then assignment */
			Ok(try!(self.parse_no_delete_expr()) || try!(self.parse_assignment()))
		}
	}
	
	fn parse_assignment(&mut self) -> Result<bool, Error>
	{
		if !try!(self.parse_index_expr(false))
		{
			return Ok(false)
		}
		
		try!(self.visitor.assign_element(self.path_kind == Absolute, self.path.as_slice()));
		
		let assign = try_eof!(self.lexer.cur_token,
			Error::from_span(self.lexer.get_source(), self.path.last().unwrap().span, "Expected a '=' to follow this string literal, but got EOF"));
		if assign.kind != lex::Assign
		{
			return Error::from_span(self.lexer.get_source(), assign.span, "Expected '='");
		}
		self.lexer.next();
		
		if !try!(self.parse_expr())
		{
			let cur_token = try_eof!(self.lexer.cur_token,
				Error::from_span(self.lexer.get_source(), assign.span, "Expected a RHS to finish this assignment, but got EOF"));
			return Error::from_span(self.lexer.get_source(), cur_token.span, "Expected an expression or 'delete'");
		}
		
		Ok(true)
	}

	fn parse_index_expr(&mut self, rhs: bool) -> Result<bool, Error>
	{
		let token = try_eof!(self.lexer.cur_token, Ok(false));
		if token.kind.is_string() || token.kind == lex::Root || (rhs && token.kind == lex::Import)
		{
			self.path.clear();
			self.path_kind = match token.kind
			{
				lex::Root => Absolute,
				lex::Import => Import,
				_ => Relative
			};
			
			if self.path_kind == Relative
			{
				self.path.push(ConfigString::from_token(token));
			}

			loop
			{
				let start_token = try_eof!(self.lexer.next(), Ok(true));
				if start_token.kind != lex::LeftBracket
				{
					return Ok(true);
				}
				
				let path_token = try_eof!(self.lexer.next(),
					Error::from_span(self.lexer.get_source(), start_token.span, "Expected a string literal to continue this index expression, but got EOF"));
				if !path_token.kind.is_string()
				{
					return Error::from_span(self.lexer.get_source(), start_token.span, "Expected a string literal");
				}
				self.path.push(ConfigString::from_token(path_token));
				
				let end_token = try_eof!(self.lexer.next(),
					Error::from_span(self.lexer.get_source(), start_token.span, "Expected a ']' to finish this index expression, but got EOF"));
				if end_token.kind != lex::RightBracket
				{
					return Error::from_span(self.lexer.get_source(), end_token.span, "Expected a ']'");
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
			let token = try_eof!(self.lexer.cur_token, Ok(false));
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
			return Ok(true);
		}
		
		let brace_or_bracket = try_eof!(self.lexer.cur_token, Ok(false));
		
		if brace_or_bracket.kind == lex::LeftBrace
		{
			self.lexer.next();
			try!(self.visitor.start_table());
			try!(self.parse_table_contents(false));
			
			let brace = try_eof!(self.lexer.cur_token, Error::from_span(self.lexer.get_source(), brace_or_bracket.span, "Unterminated table"));
			if brace.kind != lex::RightBrace
			{
				return Error::from_span(self.lexer.get_source(), brace.span, "Expected '}'");
			}
			self.lexer.next();
			
			try!(self.visitor.end_table());
			Ok(true)
		}
		else if brace_or_bracket.kind == lex::LeftBracket
		{
			self.lexer.next();
			try!(self.visitor.start_array());
			try!(self.parse_array_contents());
			
			let bracket = try_eof!(self.lexer.cur_token, Error::from_span(self.lexer.get_source(), brace_or_bracket.span, "Unterminated array"));
			if bracket.kind != lex::RightBracket
			{
				return Error::from_span(self.lexer.get_source(), bracket.span, "Expected ',' or ']'");
			}
			self.lexer.next();
			
			try!(self.visitor.end_array());
			Ok(true)
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
			if !try!(self.parse_string_source())
			{
				match last_span
				{
					Some(span) =>
					{
						return Error::from_span(self.lexer.get_source(), span, "Expected a string source to finish this concatenation, but got EOF");
					}
					None =>
					{
						return Ok(false)
					}
				}
			}
			
			let expand = try_eof!(self.lexer.cur_token, Ok(true));
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
		let token = try_eof!(self.lexer.cur_token, Ok(false));
		if token.kind.is_string()
		{
			try!(self.visitor.append_string(ConfigString::from_token(token)));
			self.lexer.next();
			Ok(true)
		}
		else if try!(self.parse_expansion())
		{
			try!(self.visitor.append_path(self.path_kind, self.path.as_slice()));
			Ok(true)
		}
		else
		{
			Ok(false)
		}
	}

	fn parse_expansion(&mut self) -> Result<bool, Error>
	{
		let token = try_eof!(self.lexer.cur_token, Ok(false));
		if token.kind == lex::Dollar
		{
			self.lexer.next();
			Ok(try!(self.parse_index_expr(true)))
		}
		else
		{
			Ok(false)
		}
	}
}

pub fn parse_source<'l, 'm, E: GetError, V: Visitor<'l, E>>(filename: &'l Path, source: &'l str, visitor: &mut V) -> Result<(), Error>
{
	let mut lexer = Lexer::new(filename, source);
	lexer.next();
	let mut parser = Parser{ lexer: lexer, visitor: visitor, path_kind: Absolute, path: vec![] };
	parser.parse_table_contents(true)
}
