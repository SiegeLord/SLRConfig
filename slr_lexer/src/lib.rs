// Copyright (c) 2014 by SiegeLord
//
// All rights reserved. Distributed under LGPL 3.0. For full terms see the file LICENSE.

use std::char::is_whitespace;
use std::str::{mod, CharOffsets};

#[deriving(Show, Clone)]
pub struct Span
{
	start: uint,
	len: uint,
}

#[deriving(Show, Clone)]
pub struct Token<'l>
{
	pub kind: TokenKind<'l>,
	pub span: Span
}

impl<'l> Token<'l>
{
	fn new(kind: TokenKind<'l>, span: Span) -> Token<'l>
	{
		Token{ kind: kind, span: span }
	}
}

#[deriving(PartialEq, Show, Clone)]
pub enum TokenKind<'l>
{
	EscapedString(&'l str),
	RawString(&'l str),
	Root,
	Import,
	Assign,
	LeftBracket,
	RightBracket,
	LeftBrace,
	RightBrace,
	Dollar,
	Comma,
	Tilde,
	Eof
}

impl<'l> TokenKind<'l>
{
	pub fn is_string(&self) -> bool
	{
		match *self
		{
			EscapedString(_) | RawString (_) => true,
			_ => false
		}
	}
}

fn is_naked_string_border(c: char) -> bool
{
	!is_whitespace(c) &&
	c != '=' &&
	c != '[' &&
	c != ']' &&
	c != '{' &&
	c != '}' &&
	c != '$' &&
	c != ',' &&
	c != '~' &&
	c != '"' &&
	c != '#'
}

fn is_naked_string_middle(c: char) -> bool
{
	is_naked_string_border(c) || c == ' '
}

fn is_newline(c: char) -> bool
{
	c == '\n' || c == '\r'
}

struct Source<'l>
{
	filename: &'l str,
	source: &'l str,
	chars: CharOffsets<'l>,
	
	cur_char: Option<char>,
	cur_pos: uint,
	
	next_char: Option<char>,
	next_pos: uint,
	
	line_start_pos: uint,
	at_newline: bool,
	ignore_next_newline: bool,
	
	line_ends: Vec<uint>,

	span_start: uint,
}

impl<'l> Source<'l>
{
	fn new(filename: &'l str, source: &'l str) -> Source<'l>
	{
		let chars = source.char_indices();
		let mut src = 
			Source
			{
				filename: filename,
				source: source,
				chars: chars,
				cur_char: None,
				cur_pos: 0,
				next_char: None,
				next_pos: 0,
				line_start_pos: 0,
				at_newline: false,
				ignore_next_newline: false,
				line_ends: vec![],
				span_start: 0,
			};
		src.bump();
		src.bump();
		src
	}
	
	fn get_line_start_end(&self, line: uint) -> (uint, uint)
	{
		if line > self.line_ends.len() + 1
		{
			fail!("Trying to get an unvisited line!");
		}
		let start = if line == 0
		{
			0
		}
		else
		{
			self.line_ends[line - 1]
		};
		let start = match self.source.slice_from(start).chars().position(|c| !is_newline(c))
		{
			Some(offset) => start + offset,
			None => self.source.len()
		};
		let end = match self.source.slice_from(start).chars().position(|c| is_newline(c))
		{
			Some(end) => end + start,
			None => self.source.len()
		};
		(start, end)
	}

	fn get_line<'l>(&'l self, line: uint) -> &'l str
	{
		let (start, end) = self.get_line_start_end(line);
		self.source.slice(start, end)
	}

	#[allow(dead_code)]
	fn get_cur_col(&self) -> uint
	{
		if self.cur_pos >= self.line_start_pos
		{
			self.cur_pos - self.line_start_pos
		}
		else
		{
			0
		}
	}

	#[allow(dead_code)]
	fn get_cur_line(&self) -> uint
	{
		self.line_ends.len()
	}

	fn start_span(&mut self)
	{
		self.span_start = self.cur_pos;
	}

	fn get_span(&self) -> Span
	{
		let len = if self.cur_pos == self.span_start
		{
			1
		}
		else
		{
			self.cur_pos - self.span_start
		};
		Span
		{
			start: self.span_start,
			len: len,
		}
	}

	fn get_line_col_from_pos(&self, pos: uint) -> (uint, uint)
	{
		use std::slice::{Found, NotFound};
		let line = match self.line_ends.as_slice().binary_search(|end| end.cmp(&pos))
		{
			Found(n) => n,
			NotFound(n) => n
		};
		let (start, _) = self.get_line_start_end(line);
		if pos < start
		{
			fail!("Position less than line start (somehow got a position inside a newline!)")
		}
		(line, pos - start)
	}

	fn bump(&mut self) -> Option<char>
	{
		self.cur_char = self.next_char;
		self.cur_pos = self.next_pos;
		
		match self.chars.next()
		{
			Some((pos, c)) =>
			{
				self.next_pos = pos;
				self.next_char = Some(c);
			},
			None =>
			{
				self.next_pos = self.source.len();
				self.next_char = None;
			},
		}
		
		self.at_newline = self.cur_char.map_or(false, |c| is_newline(c));
		
		if self.at_newline
		{
			self.line_start_pos = self.cur_pos + 1;
			if !self.ignore_next_newline
			{
				self.line_ends.push(self.cur_pos);
			}
		}
		
		self.ignore_next_newline = self.cur_char == Some('\r') && self.next_char == Some('\n');
		
		self.cur_char
	}
}

impl<'l> Iterator<char> for Source<'l>
{
	fn next(&mut self) -> Option<char>
	{
		self.bump()
	}
}

pub struct Lexer<'l>
{	
	source: Source<'l>,
	pub cur_token: Option<Result<Token<'l>, Error>>,
	pub next_token: Option<Result<Token<'l>, Error>>,
}

#[deriving(Show, Clone)]
pub struct Error
{
	pub text: String
}

impl Error
{
	pub fn new(text: String) -> Error
	{
		Error
		{
			text: text
		}
	}

	pub fn from_pos<'l, T>(lexer: &Lexer<'l>, pos: uint, msg: &str) -> Result<T, Error>
	{
		let (line, col) = lexer.source.get_line_col_from_pos(pos);

		let source = lexer.source.get_line(line);
		let mut col_str = String::with_capacity(col + 1);
		if col > 0
		{
			let num_tabs = source.slice_to(col).chars().filter(|&c| c == '\t').count();
			col_str.grow(col + num_tabs * 3, ' ');
		}
		col_str.push_char('^');
		
		let source = str::replace(source, "\t", "    ");
		Err(Error::new(format!("{}:{}:{}: error: {}\n{}\n{}\n", lexer.source.filename, line + 1, col, msg, source, col_str)))
	}

	pub fn from_span<'l, T>(lexer: &Lexer<'l>, span: Span, msg: &str) -> Result<T, Error>
	{
		let (start_line, start_col) = lexer.source.get_line_col_from_pos(span.start);
		let (end_line, end_col) = lexer.source.get_line_col_from_pos(span.start + span.len - 1);
		
		let source = lexer.source.get_line(start_line);
		let end_col = if start_line == end_line
		{
			end_col
		}
		else
		{
			source.len() - 1
		};
		
		let mut col_str = String::with_capacity(end_col);
		if start_col > 0
		{
			let num_start_tabs = source.slice_to(start_col).chars().filter(|&c| c == '\t').count();
			col_str.grow(start_col + num_start_tabs * 3, ' ');
		}
		col_str.push_char('^');
		if end_col > start_col + 1
		{
			let num_end_tabs = source.slice(start_col, end_col).chars().filter(|&c| c == '\t').count();
			col_str.grow(end_col - start_col + num_end_tabs * 3, '~');
		}
		
		let source = str::replace(source, "\t", "    ");
		Err(Error::new(format!("{}:{}:{} - {}:{}: error: {}\n{}\n{}\n", lexer.source.filename, start_line + 1, start_col, end_line + 1, end_col,
			msg, source, col_str)))
	}
}

impl<'l> Lexer<'l>
{
	pub fn new(filename: &'l str, source: &'l str) -> Lexer<'l>
	{
		let mut lex = 
			Lexer
			{
				source: Source::new(filename, source),
				cur_token: None,
				next_token: None,
			};
		lex.next();
		lex
	}

	fn skip_whitespace<'m>(&'m mut self) -> bool
	{
		if !self.source.cur_char.map_or(false, |c| is_whitespace(c))
		{
			return false;
		}
		for c in self.source
		{
			if !is_whitespace(c)
			{
				break;
			}
		}
		true
	}

	fn skip_comments<'m>(&'m mut self) -> bool
	{
		if self.source.cur_char != Some('#')
		{
			return false;
		}
		for _ in self.source
		{
			if self.source.at_newline
			{
				break;
			}
		}
		true
	}
	
	fn eat_naked_string<'m>(&'m mut self) -> Option<Result<Token<'l>, Error>>
	{
		//~ println!("naked: {}", self.source.cur_char);
		if !self.source.cur_char.map_or(false, |c| is_naked_string_border(c))
		{
			return None;
		}
		
		let start_pos = self.source.cur_pos;
		let mut end_pos = self.source.cur_pos;
		let mut last_is_border = true;
		let mut escape_next = false;
		loop
		{
			if last_is_border
			{
				end_pos = self.source.cur_pos;
			}
			
			match self.source.cur_char
			{
				Some(c) =>
				{
					if escape_next
					{
						last_is_border = true;
						escape_next = false;
					}
					else if is_naked_string_border(c)
					{
						last_is_border = true;
						if c == '\\'
						{
							escape_next = true;
						}
					}
					else if is_naked_string_middle(c)
					{
						last_is_border = false;
					}
					else
					{
						break;
					}
				}
				None =>
				{
					break;
				}
			}
			self.source.bump();
		}

		if escape_next
		{
			/* Got EOF while trying to escape it... */
			return Some(Error::from_pos(self, end_pos, "Unexpected EOF while parsing escape in naked string literal"));
		}
		
		let contents = self.source.source.slice(start_pos, end_pos);
		let span = Span{ start: start_pos, len: end_pos - start_pos };
		let kind = match contents
		{
			"root" => Root,
			"import" => Import,
			_ => EscapedString(contents),
		};
		Some(Ok(Token::new(kind, span)))
	}
	
	fn eat_raw_string<'m>(&'m mut self) -> Option<Result<Token<'l>, Error>>
	{
		if self.source.cur_char != Some('r') ||
			!(self.source.next_char == Some('"') || self.source.next_char == Some('#'))
		{
			return None;
		}
		self.source.start_span();
		// +1 to skip the leading 'r'
		let mut start_pos = self.source.cur_pos + 1;
		let mut end_pos = start_pos;
		let mut num_leading_hashes = 0u;
		for c in self.source
		{
			match c
			{
				'#' =>
				{
					num_leading_hashes += 1;
					start_pos += 1;
				},
				'"' =>
				{
					start_pos += 1;
					break;
				},
				_ => return Some(Error::from_pos(self, self.source.span_start,
					r#"Unexpected character while parsing raw string literal (expected '#' or '"')"#)),
			}
		}
		'done: for c in self.source
		{
			if c == '"'
			{
				end_pos = self.source.cur_pos;
				let mut num_trailing_hashes = 0;
				
				for c in self.source
				{
					if num_trailing_hashes == num_leading_hashes
					{
						break 'done;
					}
					match c
					{
						'#' => num_trailing_hashes += 1,
						_ => break,
					}
				}
			}
		}
		
		if self.source.cur_char.is_none()
		{
			Some(Error::from_pos(self, self.source.span_start, "Unexpected EOF while looking for the end of this raw string literal"))
		}
		else
		{
			Some(Ok(Token::new(RawString(self.source.source.slice(start_pos, end_pos)), self.source.get_span())))
		}
	}
	
	fn eat_escaped_string<'m>(&'m mut self) -> Option<Result<Token<'l>, Error>>
	{
		if self.source.cur_char != Some('"')
		{
			return None;
		}
		self.source.start_span();
		// +1 to skip the leading '"'
		let start_pos = self.source.cur_pos + 1;
		let mut last_is_slash = false;
		for c in self.source
		{
			if c == '"' && !last_is_slash
			{
				break;
			}
			last_is_slash = c == '\\' && !last_is_slash;
		}
		if self.source.cur_char.is_none()
		{
			return Some(Error::from_pos(self, self.source.span_start,
				"Unexpected EOF while looking for the end of this escaped string literal"))
		}
		let contents = self.source.source.slice(start_pos, self.source.cur_pos);
		// Skip the trailing "
		self.source.bump();
		Some(Ok(Token::new(EscapedString(contents), self.source.get_span())))
	}

	fn eat_char_tokens<'m>(&'m mut self) -> Option<Result<Token<'l>, Error>>
	{
		//~ println!("char");
		self.source.cur_char.and_then(|c|
		{
			match c
			{
				'=' => Some(Assign),
				'[' => Some(LeftBracket),
				']' => Some(RightBracket),
				'{' => Some(LeftBrace),
				'}' => Some(RightBrace),
				'$' => Some(Dollar),
				',' => Some(Comma),
				'~' => Some(Tilde),
				_ => None
			}
		}).map(|kind|
		{
			self.source.start_span();
			self.source.bump();
			Ok(Token::new(kind, self.source.get_span()))
		})
	}

	pub fn next<'m>(&'m mut self) -> Option<Result<Token<'l>, Error>>
	{
		if self.cur_token.as_ref().map_or(true, |res| res.is_ok())
		{
			while self.skip_whitespace() || self.skip_comments() {}
			self.cur_token = self.next_token.take();
			self.next_token = self.eat_char_tokens()
				.or_else(|| self.eat_raw_string())
				.or_else(|| self.eat_naked_string())
				.or_else(|| self.eat_escaped_string());
		}
		
		self.cur_token.clone()
	}
}
