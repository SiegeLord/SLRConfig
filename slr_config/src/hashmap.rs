// Copyright (c) 2014 by SiegeLord
//
// All rights reserved. Distributed under LGPL 3.0. For full terms see the file LICENSE.

use std::collections::hashmap::HashMap;

use visitor::Visitor;
use lex::Error;
use parser::{ConfigString, PathKind};

pub enum ConfigValue
{
	Leaf(String),
	Node(HashMap<String, ConfigElement>),
}

pub struct ConfigElement
{
	value: ConfigValue,
}

struct PathVector
{
	data: Vec<String>,
	len: uint,
}

impl PathVector
{
	fn new() -> PathVector
	{
		PathVector
		{
			data: vec![],
			len: 0,
		}
	}
	
	fn clear(&mut self)
	{
		self.len = 0;
	}

	fn push(&mut self, path: ConfigString)
	{
		if self.len == self.data.len()
		{
			self.data.push(String::new());
		}
		path.into_string(self.data.get_mut(self.len));
		self.len += 1;
	}
}

pub struct HashmapVisitor
{
	root: HashMap<String, ConfigElement>,
	current_path: PathVector,
	assign_path: PathVector,
	assign_path_absolute: bool,
}

impl HashmapVisitor
{
	pub fn new() -> HashmapVisitor
	{
		HashmapVisitor
		{
			assign_path: PathVector::new(),
			current_path: PathVector::new(),
			assign_path_absolute: false,
			root: HashMap::new(),
		}
	}
}

impl<'l> Visitor<'l, Error> for HashmapVisitor
{
	fn assign_element(&mut self, path: &[ConfigString<'l>]) -> Result<(), Error>
	{
		println!("Started assignment: {}", path);
		Ok(())
	}

	fn insert_path(&mut self, path_kind: PathKind, path: &[ConfigString<'l>]) -> Result<(), Error>
	{
		println!("Inserted {} path: {}", path_kind, path);
		Ok(())
	}

	fn append_path(&mut self, path_kind: PathKind, path: &[ConfigString<'l>]) -> Result<(), Error>
	{
		println!("Appended {} path: {}", path_kind, path);
		Ok(())
	}
	
	fn append_string(&mut self, string: ConfigString<'l>) -> Result<(), Error>
	{
		println!("String appended: {}", string.to_string());
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

