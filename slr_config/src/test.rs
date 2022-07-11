// Copyright (c) 2014 by SiegeLord
//
// All rights reserved. Distributed under LGPL 3.0. For full terms see the file LICENSE.

use crate::config_element::*;
use crate::de::from_element;
use crate::ser::to_element;
use serde_derive::{Deserialize, Serialize};
use std::char;

#[test]
fn basic_test()
{
	let mut root = ConfigElement::new_table();
	let val = ConfigElement::new_value("warudo");
	root.insert("za", val);
	assert!(root.as_table().is_some());
	assert!(root.as_table().unwrap()["za"].as_value().is_some());
	assert_eq!(root.as_table().unwrap()["za"].as_value().unwrap(), "warudo");
}

#[test]
fn basic_printing_test()
{
	let mut root = ConfigElement::new_table();
	let val = ConfigElement::new_value("warudo");
	root.insert("za", val);
	assert_eq!(format!("{}", root), "za = warudo\n");
}

#[test]
fn basic_parsing_test()
{
	let root = ConfigElement::from_str("za = warudo").unwrap();
	assert!(root.as_table().is_some());
	assert!(root.as_table().unwrap()["za"].as_value().is_some());
	assert_eq!(root.as_table().unwrap()["za"].as_value().unwrap(), "warudo");
}

#[test]
fn init_test()
{
	let mut root = ConfigElement::new_table();
	root.insert("za", ConfigElement::new_value("warudo"));
	root.from_str_with_init("what = $za").unwrap();
	assert_eq!(
		root.as_table().unwrap()["what"].as_value().unwrap(),
		"warudo"
	);
}

#[test]
fn roundtrip_test()
{
	let src = r#"
# Comment
val1 = {{{" "}}a"}}}
val2 = b ~ c d
r0 = ""
r2 = ""
r2 = {{""}}
r3 = {{"a"}}
r4 = {{"aa"}}
r5 = {{"""}}
r6 = {{""}"}}
r7 = {{{""}}"}}}

arr1 = []
arr3 = [a]
arr2 = [[], {}]

tag1 = a {}
tag2 = a { a = 2 }
tag3 = [a {}, b {}]

taga1 = a []
taga2 = a [1, 2]
taga3 = [a [1], b [2]]

foo2 = [a]

foo2 = test

bar
{
	bar_foo = test ~ baz
	bar_bar
	{
		foo
		{
			"bar	bar1" = ["\ttest"]
			bar bar2 = [test,]
			bar bar3 = [test,test]
			bar bar4 = []
		}
	}
}

baz
{
	bar_bar
	{
		foo = ""
		bar = "\u0021d"
	}
}
"#;
	let original = ConfigElement::from_str(src).unwrap();
	assert_eq!(
		original.as_table().unwrap()["foo2"].as_value().unwrap(),
		"test"
	);
	let original_str = format!("{}", original);
	let decoded = ConfigElement::from_str(&original_str).unwrap();
	let encoded_str = format!("{}", decoded);
	assert_eq!(original_str, encoded_str);
}

#[test]
fn unicode_encode_test()
{
	for i in 0..1000u32
	{
		let s = format!("{}", char::from_u32(i).unwrap());
		let mut root = ConfigElement::new_table();
		root.insert("test", ConfigElement::new_value(&s));
		let encoded = format!("{}", root);
		println!("Encoding: {} |{}|\n{}", i, s, encoded);
		let decoded = ConfigElement::from_str(&encoded)
			.map_err(|e| print!("{}", e.text))
			.unwrap();
		assert_eq!(&s, decoded.as_table().unwrap()["test"].as_value().unwrap());
	}
}

#[test]
fn expand_test()
{
	let src = r#"
val = "a"
arr = []
tab {}

val_test = $val ~ $val
arr_test = $arr
tab_test = $tab

tab2
{
	tab = "b"
	val_test2 = $tab
}
"#;
	let root = ConfigElement::from_str(src).unwrap();
	let root = root.as_table().unwrap();
	assert!(root["val_test"].as_value().is_some());
	assert!(root["val_test"].as_value().unwrap() == "aa");
	assert!(root["arr_test"].as_array().is_some());
	assert!(root["tab_test"].as_table().is_some());
	assert!(root["tab2"].as_table().unwrap()["val_test2"]
		.as_value()
		.is_some());
}

#[test]
fn serde_test()
{
	use slr_parser::Source;
	use std::collections::HashMap;
	use std::path::Path;

	#[derive(Serialize, Deserialize, PartialEq, Debug)]
	struct Unit;

	#[derive(Serialize, Deserialize, PartialEq, Debug)]
	struct NewType(i32);

	#[derive(Serialize, Deserialize, PartialEq, Debug)]
	struct Tuple(i32, i32);

	#[derive(Serialize, Deserialize, PartialEq, Debug)]
	struct B
	{
		a: i32,
	}

	#[derive(Serialize, Deserialize, PartialEq, Debug)]
	struct A
	{
		b: i32,
		c: Option<i32>,
		d: Option<f32>,
		e: Vec<E>,
		f: HashMap<i32, i32>,
		h: (i32, i32),
		g1: B,
		g2: B,
		i: Unit,
		j1: NewType,
		j2: NewType,
		k1: Tuple,
		k2: Tuple,
	}

	#[derive(Serialize, Deserialize, PartialEq, Debug)]
	enum E
	{
		Var1,
		Var2(i32),
		Var3
		{
			v: i32,
		},
		Var4(i32, i32),
	}

	let mut f = HashMap::new();
	f.insert(1, 2);
	let v = A {
		b: 1,
		c: None,
		d: Some(1.0),
		e: vec![E::Var1, E::Var2(1), E::Var3 { v: 1 }, E::Var4(1, 2)],
		f: f,
		h: (1, 2),
		g1: B { a: 1 },
		g2: B { a: 2 },
		i: Unit,
		j1: NewType(1),
		j2: NewType(2),
		k1: Tuple(1, 1),
		k2: Tuple(2, 2),
	};

	let elem = to_element(&v).unwrap();
	println!("\n{}", elem);

	let src_str = r#"
		b = 1
		c = ""
		d = 1
		e =
		[
				Var1,
				Var2 [1],
				Var3
				{
					v = 1
				},
				Var4 [1, 2]
		]
		f = [[1, 2]]
		g1 = B
		{
			a = 1
		}
		g2
		{
			a = 2
		}
		h = [1, 2]
		i = Unit
		j1 = [1]
		j2 = NewType [2]
		k1 = [1, 1]
		k2 = Tuple [2, 2]
	"#;
	let mut src = Source::new(&Path::new("none"), &src_str);
	let elem = ConfigElement::from_source(&mut src);
	if let Err(ref err) = elem
	{
		println!("Error");
		println!("{}", err.text);
	}
	let elem = elem.unwrap();

	let v2 = from_element(&elem, Some(&src));

	if let Err(ref err) = v2
	{
		println!("Error");
		println!("{}", err.text);
	}
	let v2 = v2.unwrap();

	assert_eq!(v, v2);
}
