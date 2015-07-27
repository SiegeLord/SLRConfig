// Copyright (c) 2014 by SiegeLord
//
// All rights reserved. Distributed under LGPL 3.0. For full terms see the file LICENSE.

#[cfg(test)]
use config_element::*;
#[cfg(test)]
use element_repr::*;
#[cfg(test)]
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
	assert_eq!(root.as_table().unwrap()["what"].as_value().unwrap(), "warudo");
}

#[test]
fn roundtrip_test()
{
	let src =
r#"
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
	assert_eq!(original.as_table().unwrap()["foo2"].as_value().unwrap(), "test");
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
		let decoded = ConfigElement::from_str(&encoded).map_err(|e| print!("{}", e.text)).unwrap();
		assert_eq!(&s, decoded.as_table().unwrap()["test"].as_value().unwrap());
	}
}

#[test]
fn expand_test()
{
	let src =
r#"
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
	assert!(root["tab2"].as_table().unwrap()["val_test2"].as_value().is_some());
}

#[test]
fn from_element_test()
{
	let elem = ConfigElement::new_value("55");
	let mut val: i32 = 0;
	val.from_element(&elem, None).unwrap();
	assert_eq!(val, 55);

	let mut elem = ConfigElement::new_array();
	elem.insert("", ConfigElement::new_value("55"));
	let mut val: Vec<i32> = vec![];
	val.from_element(&elem, None).unwrap();
	assert_eq!(val[0], 55);
}

#[test]
fn slr_struct()
{
	slr_def!
	{
		#[derive(Clone)]
		struct Test
		{
			x: i32 = 0,
			y: i32 = 0
		}
	}
	let orig = Test::new();

	let mut empty_test = orig.clone();
	let empty_test_elem = ConfigElement::new_table();
	empty_test.from_element(&empty_test_elem, None).unwrap();
	assert_eq!(empty_test.x, 0);
	assert_eq!(empty_test.y, 0);

	let mut partial_test = orig.clone();
	let mut partial_test_elem = ConfigElement::new_table();
	partial_test_elem.insert("x", ConfigElement::new_value("5"));
	partial_test.from_element(&partial_test_elem, None).unwrap();
	assert_eq!(partial_test.x, 5);
	assert_eq!(partial_test.y, 0);

	let partial_test_elem = partial_test.to_element();
	assert!(partial_test_elem.as_table().is_some());
	assert_eq!(partial_test_elem.as_table().unwrap()["x"].as_value().unwrap(), "5");
	assert_eq!(partial_test_elem.as_table().unwrap()["y"].as_value().unwrap(), "0");
}

#[test]
fn slr_enum()
{
	slr_def!
	{
		#[derive(Clone, PartialEq, Debug)]
		enum Test
		{
			A,
			B
		}
	}
	let mut test = Test::A;

	let test_elem = ConfigElement::new_value("B");
	test.from_element(&test_elem, None).unwrap();
	assert_eq!(test, Test::B);

	let test_elem = test.to_element();
	assert_eq!(test_elem.as_value().unwrap(), "B");
}
