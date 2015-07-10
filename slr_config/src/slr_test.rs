// This file is released into Public Domain.

extern crate slr_config;

use std::env;
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;
use slr_config::{HashmapVisitor, parse_source};

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
	
	let mut visitor = HashmapVisitor::new();
	parse_source(Path::new(&filename), &src, &mut visitor).map_err(|e| print!("{}", e.text)).unwrap();
}
