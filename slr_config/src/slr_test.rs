// This file is released into Public Domain.

extern crate slr_config;

use std::os;
use std::io::File;
use std::path::Path;
use slr_config::{HashmapVisitor, parse_source};

fn main()
{
	let args = os::args();
	if args.len() < 2
	{
		fail!("Pass a file to test with");
	}
	
	let filename = Path::new(args[1].as_slice());
	
	let src = File::open(&filename).unwrap().read_to_string().unwrap();
	
	let mut visitor = HashmapVisitor::new();
	parse_source(&filename, src.as_slice(), &mut visitor).map_err(|e| print!("{}", e.text));
}
