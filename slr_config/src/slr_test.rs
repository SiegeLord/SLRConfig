// This file is released into Public Domain.

extern crate slr_config;

use std::env;
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;
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

	let (root, _) = ConfigElement::from_str(Path::new(&filename), &src).map_err(|e| print!("{}", e.text)).unwrap();
	
	println!("{}", root);
	println!("{}", root.as_table().unwrap()["val1"].as_value().unwrap());
}
