use config_element::ConfigElement;
use serde;
use serde::ser::{self, Serialize};
use slr_parser::Error;

/// Serialize a value to a ConfigElement.
pub fn to_element<T: Serialize>(value: &T) -> Result<ConfigElement, Error>
{
	value.serialize(Serializer)
}

struct SeqHelper
{
	element: ConfigElement,
}

impl SeqHelper
{
	fn new_array() -> Self
	{
		Self {
			element: ConfigElement::new_array(),
		}
	}

	fn new_tagged_table(name: &str) -> Self
	{
		Self {
			element: ConfigElement::new_tagged_table(name.to_string()),
		}
	}
}

impl ser::SerializeSeq for SeqHelper
{
	type Ok = ConfigElement;
	type Error = Error;

	fn serialize_element<T>(&mut self, value: &T) -> Result<(), Error>
	where
		T: ?Sized + Serialize,
	{
		self.element.insert("", value.serialize(Serializer)?);
		Ok(())
	}

	fn end(self) -> Result<ConfigElement, Error>
	{
		Ok(self.element)
	}
}

impl ser::SerializeTuple for SeqHelper
{
	type Ok = ConfigElement;
	type Error = Error;

	fn serialize_element<T>(&mut self, value: &T) -> Result<(), Error>
	where
		T: ?Sized + Serialize,
	{
		self.element.insert("", value.serialize(Serializer)?);
		Ok(())
	}

	fn end(self) -> Result<ConfigElement, Error>
	{
		Ok(self.element)
	}
}

impl ser::SerializeTupleStruct for SeqHelper
{
	type Ok = ConfigElement;
	type Error = Error;

	fn serialize_field<T>(&mut self, value: &T) -> Result<(), Error>
	where
		T: ?Sized + Serialize,
	{
		self.element.insert("", value.serialize(Serializer)?);
		Ok(())
	}

	fn end(self) -> Result<ConfigElement, Error>
	{
		Ok(self.element)
	}
}

impl ser::SerializeStruct for SeqHelper
{
	type Ok = ConfigElement;
	type Error = Error;

	fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Error>
	where
		T: ?Sized + Serialize,
	{
		self.element.insert(key, value.serialize(Serializer)?);
		Ok(())
	}

	fn end(self) -> Result<ConfigElement, Error>
	{
		Ok(self.element)
	}
}

struct MapHelper
{
	key: Option<ConfigElement>,
	element: ConfigElement,
}

impl MapHelper
{
	fn new() -> Self
	{
		Self {
			key: None,
			element: ConfigElement::new_array(),
		}
	}
}

impl ser::SerializeMap for MapHelper
{
	type Ok = ConfigElement;
	type Error = Error;

	fn serialize_key<T>(&mut self, key: &T) -> Result<(), Error>
	where
		T: ?Sized + Serialize,
	{
		self.key = Some(key.serialize(Serializer)?);
		Ok(())
	}

	fn serialize_value<T>(&mut self, value: &T) -> Result<(), Error>
	where
		T: ?Sized + Serialize,
	{
		let key = self.key.take().unwrap();
		let value = value.serialize(Serializer)?;

		let mut pair = ConfigElement::new_array();
		pair.insert("", key);
		pair.insert("", value);
		self.element.insert("", pair);

		Ok(())
	}

	fn end(self) -> Result<ConfigElement, Error>
	{
		Ok(self.element)
	}
}

struct VariantHelper
{
	variant: &'static str,
	element: ConfigElement,
}

impl VariantHelper
{
	fn new_array(variant: &'static str) -> Self
	{
		Self {
			variant: variant,
			element: ConfigElement::new_array(),
		}
	}

	fn new_tagged_table(variant: &'static str) -> Self
	{
		Self {
			variant: variant,
			element: ConfigElement::new_tagged_table(variant.to_string()),
		}
	}
}

impl ser::SerializeTupleVariant for VariantHelper
{
	type Ok = ConfigElement;
	type Error = Error;

	fn serialize_field<T>(&mut self, value: &T) -> Result<(), Error>
	where
		T: ?Sized + Serialize,
	{
		self.element.insert("", value.serialize(Serializer)?);
		Ok(())
	}

	fn end(self) -> Result<ConfigElement, Error>
	{
		let mut ret = ConfigElement::new_table();
		ret.insert(self.variant, self.element);
		Ok(ret)
	}
}

impl ser::SerializeStructVariant for VariantHelper
{
	type Ok = ConfigElement;
	type Error = Error;

	fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Error>
	where
		T: ?Sized + Serialize,
	{
		self.element.insert(key, value.serialize(Serializer)?);
		Ok(())
	}

	fn end(self) -> Result<ConfigElement, Error>
	{
		Ok(self.element)
	}
}

struct Serializer;

impl serde::Serializer for Serializer
{
	type Ok = ConfigElement;
	type Error = Error;

	type SerializeSeq = SeqHelper;
	type SerializeTuple = SeqHelper;
	type SerializeTupleStruct = SeqHelper;
	type SerializeTupleVariant = VariantHelper;
	type SerializeMap = MapHelper;
	type SerializeStruct = SeqHelper;
	type SerializeStructVariant = VariantHelper;

	fn serialize_str(self, v: &str) -> Result<ConfigElement, Error>
	{
		Ok(ConfigElement::new_value(v))
	}

	fn serialize_bool(self, v: bool) -> Result<ConfigElement, Error>
	{
		Ok(ConfigElement::new_value(if v { "true" } else { "false" }))
	}

	fn serialize_i8(self, v: i8) -> Result<ConfigElement, Error>
	{
		Ok(ConfigElement::new_value(&v))
	}

	fn serialize_i16(self, v: i16) -> Result<ConfigElement, Error>
	{
		Ok(ConfigElement::new_value(&v))
	}

	fn serialize_i32(self, v: i32) -> Result<ConfigElement, Error>
	{
		Ok(ConfigElement::new_value(&v))
	}

	fn serialize_i64(self, v: i64) -> Result<ConfigElement, Error>
	{
		Ok(ConfigElement::new_value(&v))
	}

	fn serialize_u8(self, v: u8) -> Result<ConfigElement, Error>
	{
		Ok(ConfigElement::new_value(&v))
	}

	fn serialize_u16(self, v: u16) -> Result<ConfigElement, Error>
	{
		Ok(ConfigElement::new_value(&v))
	}

	fn serialize_u32(self, v: u32) -> Result<ConfigElement, Error>
	{
		Ok(ConfigElement::new_value(&v))
	}

	fn serialize_u64(self, v: u64) -> Result<ConfigElement, Error>
	{
		Ok(ConfigElement::new_value(&v))
	}

	fn serialize_f32(self, v: f32) -> Result<ConfigElement, Error>
	{
		Ok(ConfigElement::new_value(&v))
	}

	fn serialize_f64(self, v: f64) -> Result<ConfigElement, Error>
	{
		Ok(ConfigElement::new_value(&v))
	}

	fn serialize_char(self, v: char) -> Result<ConfigElement, Error>
	{
		Ok(ConfigElement::new_value(&v))
	}

	fn serialize_bytes(self, v: &[u8]) -> Result<ConfigElement, Error>
	{
		let mut ret = ConfigElement::new_array();
		for e in v
		{
			ret.insert("", ConfigElement::new_value(e));
		}
		Ok(ret)
	}

	fn serialize_none(self) -> Result<ConfigElement, Error>
	{
		Ok(ConfigElement::new_value(""))
	}

	fn serialize_some<T>(self, v: &T) -> Result<ConfigElement, Error>
	where
		T: ?Sized + Serialize,
	{
		v.serialize(self)
	}

	fn serialize_unit(self) -> Result<ConfigElement, Error>
	{
		Ok(ConfigElement::new_value(""))
	}

	fn serialize_unit_struct(self, name: &'static str) -> Result<ConfigElement, Error>
	{
		Ok(ConfigElement::new_value(name))
	}

	fn serialize_unit_variant(
		self, _name: &'static str, _index: u32, variant: &'static str,
	) -> Result<ConfigElement, Error>
	{
		Ok(ConfigElement::new_value(variant))
	}

	fn serialize_newtype_struct<T>(self, _name: &'static str, v: &T) -> Result<ConfigElement, Error>
	where
		T: ?Sized + Serialize,
	{
		v.serialize(self)
	}

	fn serialize_newtype_variant<T>(
		self, _name: &'static str, _variant_index: u32, variant: &'static str, value: &T,
	) -> Result<ConfigElement, Error>
	where
		T: ?Sized + Serialize,
	{
		let mut ret = ConfigElement::new_table();
		ret.insert(variant, value.serialize(Serializer)?);
		Ok(ret)
	}

	fn serialize_seq(self, _len: Option<usize>) -> Result<SeqHelper, Error>
	{
		Ok(SeqHelper::new_array())
	}

	fn serialize_tuple(self, _len: usize) -> Result<SeqHelper, Error>
	{
		Ok(SeqHelper::new_array())
	}

	fn serialize_tuple_struct(self, _name: &'static str, _len: usize) -> Result<SeqHelper, Error>
	{
		Ok(SeqHelper::new_array())
	}

	fn serialize_tuple_variant(
		self, _name: &'static str, _variant_index: u32, variant: &'static str, _len: usize,
	) -> Result<VariantHelper, Error>
	{
		Ok(VariantHelper::new_array(variant))
	}

	fn serialize_map(self, _len: Option<usize>) -> Result<MapHelper, Error>
	{
		Ok(MapHelper::new())
	}

	fn serialize_struct(self, name: &'static str, _len: usize) -> Result<SeqHelper, Error>
	{
		Ok(SeqHelper::new_tagged_table(name))
	}

	fn serialize_struct_variant(
		self, _name: &'static str, _variant_index: u32, variant: &'static str, _len: usize,
	) -> Result<VariantHelper, Error>
	{
		Ok(VariantHelper::new_tagged_table(variant))
	}
}
