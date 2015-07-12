// This file is released into Public Domain.

extern crate slr_config;

use std::env;
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;
use slr_config::{ConfigElement, Value, Table, Array};

pub fn print_element(depth: usize, elem: &ConfigElement)
{
	let mut indent = String::new();
	for _ in 0..depth
	{
		indent.push_str("  ");
	}

	match elem.kind
	{
		Value(ref val) => println!("{}", val),
		Table(ref table) =>
		{
			println!("\n{}{{", indent);
			for (k, v) in table.iter()
			{
				print!("{}  {} = ", indent, k);
				print_element(depth + 2, v);
			}
			println!("{}}}", indent);
		}
		Array(ref array) =>
		{
			println!("\n{}[", indent);
			for v in array
			{
				print_element(depth + 1, v);
			}
			println!("{}]", indent);
		}
	}
}

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
	
	print_element(0, &root);
}
