// Copyright (c) 2014 by SiegeLord
//
// All rights reserved. Distributed under LGPL 3.0. For full terms see the file LICENSE.

#[cfg(test)]
use config_element::*;
#[cfg(test)]
use std::path::Path;
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
	let (root, _) = ConfigElement::from_str(Path::new("<dummy>"), "za = warudo").unwrap();
	assert!(root.as_table().is_some());
	assert!(root.as_table().unwrap()["za"].as_value().is_some());
	assert_eq!(root.as_table().unwrap()["za"].as_value().unwrap(), "warudo");
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
	let (original, _) = ConfigElement::from_str(Path::new("<dummy>"), src).unwrap();
	assert_eq!(original.as_table().unwrap()["foo2"].as_value().unwrap(), "test");
	let original_str = format!("{}", original);
	let (decoded, _) = ConfigElement::from_str(Path::new("<dummy>"), &original_str).unwrap();
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
		let (decoded, _) = ConfigElement::from_str(Path::new("<dummy>"), &encoded).map_err(|e| print!("{}", e.text)).unwrap();
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
	let (root, _) = ConfigElement::from_str(Path::new("<dummy>"), src).unwrap();
	let root = root.as_table().unwrap();
	assert!(root["val_test"].as_value().is_some());
	assert!(root["val_test"].as_value().unwrap() == "aa");
	assert!(root["arr_test"].as_array().is_some());
	assert!(root["tab_test"].as_table().is_some());
	assert!(root["tab2"].as_table().unwrap()["val_test2"].as_value().is_some());
}
