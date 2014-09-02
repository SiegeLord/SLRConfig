// Copyright (c) 2014 by SiegeLord
//
// All rights reserved. Distributed under LGPL 3.0. For full terms see the file LICENSE.

use std::collections::hashmap::HashMap;

use visitor::Visitor;
use lex::Error;
use parser::ConfigString;

impl<'l> Visitor<'l, Error> for ()
{
	fn assign_element(&mut self, path: &[ConfigString<'l>]) -> Result<(), Error>
	{
		println!("Started assignment: {}", path);
		Ok(())
	}

	fn insert_path(&mut self, path: &[ConfigString<'l>]) -> Result<(), Error>
	{
		println!("Inserted path: {}", path);
		Ok(())
	}

	fn append_path(&mut self, path: &[ConfigString<'l>]) -> Result<(), Error>
	{
		println!("Path appended: {}", path);
		Ok(())
	}
	
	fn append_string(&mut self, string: ConfigString<'l>) -> Result<(), Error>
	{
		println!("String appended: {}", string);
		Ok(())
	}

	fn start_table(&mut self) -> Result<(), Error>
	{
		println!("Started table");
		Ok(())
	}
	
	fn end_table(&mut self) -> Result<(), Error>
	{
		println!("Ended table");
		Ok(())
	}

	fn start_array(&mut self) -> Result<(), Error>
	{
		println!("Started array");
		Ok(())
	}
	
	fn end_array(&mut self) -> Result<(), Error>
	{
		println!("Ended array");
		Ok(())
	}

	fn array_element(&mut self) -> Result<(), Error>
	{
		println!("Array element");
		Ok(())
	}

	fn delete(&mut self) -> Result<(), Error>
	{
		println!("Delete");
		Ok(())
	}
}
