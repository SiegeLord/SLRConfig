// This file is released into Public Domain.

extern crate slr_lexer;

use slr_lexer::Lexer;
use std::io::prelude::*;
use std::env;
use std::fs::File;
use std::path::Path;

fn main()
{
	let mut args = env::args();
	if args.len() < 2
	{
		panic!("Pass a file to test with");
	}

	args.next();
	let filename = args.next().unwrap();

	let mut src = String::new();
	File::open(&filename).unwrap().read_to_string(&mut src).unwrap();
	
	let mut lexer = Lexer::new(Path::new(&filename), &src);
	
	loop
	{
		let tok = lexer.next();
		match tok.as_ref()
		{
			Some(res) =>
			{
				match res.as_ref()
				{
					Ok(tok) => println!("{:?}", tok.kind),
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
