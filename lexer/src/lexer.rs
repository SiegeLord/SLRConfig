use std::char::is_whitespace;
use std::str::CharOffsets;

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

struct Source<'l>
{
	source: &'l str,
	chars: CharOffsets<'l>,
	
	cur_char: Option<char>,
	cur_pos: uint,
	
	next_char: Option<char>,
	next_pos: uint,
	
	at_newline: bool,
	ignore_next_newline: bool,
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
				at_newline: false,
				ignore_next_newline: false,
			};
		src.bump();
		src.bump();
		src
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
		
		self.at_newline = self.cur_char == Some('\n') || self.cur_char == Some('\r');
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
		// +1 to skip the leading 'r'
		let mut start_pos = self.source.cur_pos + 1;
		let mut end_pos;
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
				_ => return Some(Err(Error::new("Unexpected character".to_string()))),
			}
		}
		'done: loop
		{
			match self.source.bump()
			{
				Some('"') =>
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
				},
				None => return Some(Err(Error::new("Unexpected EOF".to_string()))),
				_ => (),
			}
		}
		
		Some(Ok(Token{ kind: RawString(self.source.source.slice(start_pos, end_pos)) }))
	}
	
	fn eat_escaped_string<'m>(&'m mut self) -> Option<Result<Token<'l>, Error>>
	{
		if self.source.cur_char != Some('"')
		{
			return None;
		}
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
			return Some(Err(Error::new("Unexpected EOF".to_string())));
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
	
	
	r" " = root["heh"]
	
	
	"#######;
	let mut lexer = Lexer::new(src);
	
	loop
	{
		let tok = lexer.advance_token();
		println!("{}", tok);
		if tok.as_ref().map_or(true, |res| res.is_err())
		{
			break;
		}
	}
}
