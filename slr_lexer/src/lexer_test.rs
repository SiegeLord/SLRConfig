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
		panic!("Pass a file to test with");
	}
	
	let filename = Path::new(args[1].as_slice());
	
	let src = File::open(&filename).unwrap().read_to_string().unwrap();
	
	let mut lexer = Lexer::new(&filename, src.as_slice());
	
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
