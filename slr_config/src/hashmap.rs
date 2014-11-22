// Copyright (c) 2014 by SiegeLord
//
// All rights reserved. Distributed under LGPL 3.0. For full terms see the file LICENSE.

use std::collections::HashMap;
use std::slice::Items;

use visitor::Visitor;
use lex::Error;
use parser::{ConfigString, PathKind};

pub use self::ConfigElementKind::*;

pub enum ConfigElementKind
{
	Value(String),
	Table(HashMap<String, ConfigElement>),
}

pub struct ConfigElement
{
	kind: ConfigElementKind,
}

impl ConfigElement
{
	pub fn new_table() -> ConfigElement
	{
		ConfigElement
		{
			kind: Table(HashMap::new()),
		}
	}

	pub fn new_value() -> ConfigElement
	{
		ConfigElement
		{
			kind: Value(String::new()),
		}
	}
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

	fn len(&self) -> uint
	{
		self.len
	}

	fn shrink(&mut self, new_len: uint)
	{
		if new_len < self.len
		{
			self.len = new_len;
		}
	}

	fn push_cfg_string(&mut self, path: &ConfigString)
	{
		if self.len == self.data.len()
		{
			self.data.push(String::new());
		}
		let dest = &mut self.data[self.len];
		dest.clear();
		path.into_string(dest);
		self.len += 1;
	}
	
	fn push_string(&mut self, path: &String)
	{
		if self.len == self.data.len()
		{
			self.data.push(String::new());
		}
		let dest = &mut self.data[self.len];
		dest.clear();
		dest.push_str(path.as_slice());;
		self.len += 1;
	}

	fn iter<'l>(&'l self) -> Items<'l, String>
	{
		self.data.slice_to(self.len).iter()
	}

	fn as_slice<'l>(&'l self) -> &'l [String]
	{
		self.data.slice_to(self.len)
	}
}

pub struct HashmapVisitor
{
	root: HashMap<String, ConfigElement>,
	current_path: PathVector,
	assign_path: PathVector,
	scope_stops: Vec<uint>,
	current_element: ConfigElement,
}

impl HashmapVisitor
{
	pub fn new() -> HashmapVisitor
	{
		HashmapVisitor
		{
			assign_path: PathVector::new(),
			current_path: PathVector::new(),
			scope_stops: vec![],
			root: HashMap::new(),
			current_element: ConfigElement::new_value()
		}
	}
}

impl<'l> Visitor<'l, Error> for HashmapVisitor
{
	fn assign_element(&mut self, path: &[ConfigString<'l>]) -> Result<(), Error>
	{
		self.assign_path.clear();
		for path_element in path.iter()
		{
			self.assign_path.push_cfg_string(path_element);
		}

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
		self.scope_stops.push(self.current_path.len());
		for path_entry in self.assign_path.iter()
		{
			self.current_path.push_string(path_entry);
		}
		println!("Started table. Scope: {}", self.current_path.as_slice());
		Ok(())
	}
	
	fn end_table(&mut self) -> Result<(), Error>
	{
		let old_stop = self.scope_stops.pop().unwrap_or(0);
		self.current_path.shrink(old_stop);
		println!("Ended table. Scope: {}", self.current_path.as_slice());
		Ok(())
	}

	fn start_array(&mut self) -> Result<(), Error>
	{
		self.scope_stops.push(self.current_path.len());
		for path_entry in self.assign_path.iter()
		{
			self.current_path.push_string(path_entry);
		}
		println!("Started array. Scope: {}", self.current_path.as_slice());
		Ok(())
	}
	
	fn end_array(&mut self) -> Result<(), Error>
	{
		let old_stop = self.scope_stops.pop().unwrap_or(0);
		self.current_path.shrink(old_stop);
		println!("Ended array. Scope: {}", self.current_path.as_slice());
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

