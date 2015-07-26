// This file is released into Public Domain.

extern crate slr_lexer;
extern crate slr_config;

use std::env;
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;
use slr_lexer::Source;
use slr_config::ConfigElement;

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

	let mut src = Source::new(&Path::new(&filename), &src);
	let root = ConfigElement::from_source(&mut src).map_err(|e| print!("{}", e.text)).unwrap();
	
	println!("{}", root);
}
