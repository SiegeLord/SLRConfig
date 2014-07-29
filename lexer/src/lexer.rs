use std::char::is_whitespace;
use std::str::{mod, CharOffsets};

#[deriving(Show, Clone)]
struct Token<'l>
{
	kind: TokenKind<'l>
}

#[deriving(PartialEq, Show, Clone)]
enum TokenKind<'l>
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
}

impl<'l> Source<'l>
{
	fn new(source: &'l str) -> Source<'l>
	{
		let chars = source.char_indices();
		let mut src = 
			Source
			{
				source: source,
				chars: chars,
				cur_char: None,
				cur_pos: 0,
				next_char: None,
				next_pos: 0,
				line_start_pos: 0,
				at_newline: false,
				ignore_next_newline: false,
				line_ends: vec![]
			};
		src.bump();
		src.bump();
		src
	}

	fn get_line<'l>(&'l self, line: uint) -> &'l str
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

		self.source.slice(start, end)
	}

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
	
	fn get_cur_line(&self) -> uint
	{
		self.line_ends.len()
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

struct Lexer<'l>
{	
	source: Source<'l>,
	cur_token: Option<Result<Token<'l>, Error>>,
	next_token: Option<Result<Token<'l>, Error>>,
}

#[deriving(Show, Clone)]
struct Error
{
	text: String
}

impl Error
{
	fn new(text: String) -> Error
	{
		Error
		{
			text: text
		}
	}
}

impl<'l> Lexer<'l>
{
	fn new(source: &'l str) -> Lexer<'l>
	{
		let mut lex = 
			Lexer
			{
				source: Source::new(source),
				cur_token: None,
				next_token: None,
			};
		lex.advance_token();
		lex
	}

	fn error(&self, line: uint, col: uint, msg: &str) -> Error
	{
		let source = self.source.get_line(line);
		let num_tabs = source.slice_to(col).chars().filter(|&c| c == '\t').count();
		let mut col_str = String::with_capacity(col + 1);
		if col > 0
		{
			col_str.grow(col + num_tabs * 3, ' ');
		}
		col_str.push_char('^');
		
		let source = str::replace(source, "\t", "    ");
		Error::new(format!("{}:{}: error: {}\n{}\n{}\n", line + 1, col, msg, source, col_str))
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
		self.source.bump();
		let mut end_pos = self.source.cur_pos;
		let mut last_is_border = true;
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
					if is_naked_string_border(c)
					{
						last_is_border = true;
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
		
		let contents = self.source.source.slice(start_pos, end_pos);
		Some(Ok(match contents
		{
			"root" => Token{ kind: Root },
			"import" => Token{ kind: Import },
			_ => Token{ kind: RawString(contents) }
		}))
	}
	
	fn eat_raw_string<'m>(&'m mut self) -> Option<Result<Token<'l>, Error>>
	{
		if self.source.cur_char != Some('r') ||
			!(self.source.next_char == Some('"') || self.source.next_char == Some('#'))
		{
			return None;
		}
		let start_col = self.source.get_cur_col();
		let start_line = self.source.get_cur_line();
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
				_ => return Some(Err(self.error(self.source.get_cur_line(), self.source.get_cur_col(),
					r#"Unexpected character while parsing raw string literal (expected '#' or '"')"#))),
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
			Some(Err(self.error(start_line, start_col, "Unexpected EOF while looking for the end of this raw string literal")))
		}
		else
		{
			Some(Ok(Token{ kind: RawString(self.source.source.slice(start_pos, end_pos)) }))
		}
	}
	
	fn eat_escaped_string<'m>(&'m mut self) -> Option<Result<Token<'l>, Error>>
	{
		if self.source.cur_char != Some('"')
		{
			return None;
		}
		let start_col = self.source.get_cur_col();
		let start_line = self.source.get_cur_line();
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
			return Some(Err(self.error(start_line, start_col, "Unexpected EOF while looking for the end of this escaped string literal")))
		}
		let contents = self.source.source.slice(start_pos, self.source.cur_pos);
		// Skip the trailing "
		self.source.bump();
		Some(Ok(Token{ kind: EscapedString(contents) }))
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
			self.source.bump();
			Ok(Token{ kind: kind })
		})
	}

	fn advance_token<'m>(&'m mut self) -> Option<Result<Token<'l>, Error>>
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

fn main()
{
	let src = r#######"
	
	r#" #"# = root["heh"]
	
	
	"#######;
	let mut lexer = Lexer::new(src);
	
	loop
	{
		let tok = lexer.advance_token();
		if tok.as_ref().map_or(true, |res| { res.as_ref().map_err(|err| print!("{}", err.text)).ok(); res.is_err() })
		{
			break;
		}
		println!("{}", tok);
	}
}
