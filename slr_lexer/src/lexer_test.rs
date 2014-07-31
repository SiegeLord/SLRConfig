// This file is released into Public Domain.

extern crate slr_lexer;

use slr_lexer::Lexer;
use std::os;
use std::io::File;
use std::path::Path;

fn main()
{
	let args = os::args();
	if args.len() < 2
	{
		fail!("Pass a file to test with");
	}
	
	let filename = args[1].as_slice();
	
	let src = File::open(&Path::new(filename)).unwrap().read_to_string().unwrap();
	
	let mut lexer = Lexer::new(filename.as_slice(), src.as_slice());
	
	loop
	{
		let tok = lexer.next();
		match tok.as_ref()
		{
			Some(res) =>
			{
				match res.as_ref()
				{
					Ok(tok) => println!("{}", tok.kind),
					Err(err) =>
					{
						println!("{}", err.text);
						break;
					}
				}
			}
			None => break
		}
	}
}
