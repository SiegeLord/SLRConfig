

use config_element::{ConfigElement, ConfigElementKind};
use serde::de::{self, Deserialize, Visitor};
use slr_parser::{Error, ErrorKind, Source, Span};
use std::collections::BTreeMap;
use std::collections::btree_map;
use std::error;
use std::str::FromStr;

/// Deserialize a value to a ConfigElement.
pub fn from_element<'de, 'src: 'de, T>(element: &'de ConfigElement, source: Option<&'de Source<'src>>) -> Result<T, Error>
	where T: Deserialize<'de>
{
	let d = Deserializer::new(element, source);
	T::deserialize(d)
}

struct SeqHelper<'de, 'src: 'de>
{
	elements: &'de Vec<ConfigElement>,
	source: Option<&'de Source<'src>>,
	idx: usize,
}

impl<'de, 'src> SeqHelper<'de, 'src>
{
	fn new(elements: &'de Vec<ConfigElement>, source: Option<&'de Source<'src>>) -> Self
	{
		Self {
			elements: elements,
			source: source,
			idx: 0,
		}
	}
}

impl<'de, 'src> de::SeqAccess<'de> for SeqHelper<'de, 'src>
{
	type Error = Error;

	fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Error>
		where T: de::DeserializeSeed<'de>
	{
		if self.idx < self.elements.len()
		{
			let elem = &self.elements[self.idx];
			let ret = seed.deserialize(Deserializer::new(elem, self.source))
				.map(Some);
			self.idx += 1;
			ret
		}
		else
		{
			Ok(None)
		}
	}

	fn size_hint(&self) -> Option<usize>
	{
		Some(self.elements.len())
	}
}

impl<'de, 'src> de::MapAccess<'de> for SeqHelper<'de, 'src>
{
	type Error = Error;

	fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Error>
		where K: de::DeserializeSeed<'de>
	{
		if self.idx < self.elements.len()
		{
			let elem = &self.elements[self.idx];
			if let Some(array) = elem.as_array()
			{
				if array.len() == 2
				{
					seed.deserialize(Deserializer::new(&array[0], self.source))
						.map(Some)
				}
				else
				{
					Err(Error::from_span(elem.span(), self.source, ErrorKind::InvalidRepr, "Expected a 2 element array."))
				}
			}
			else
			{
				Err(Error::from_span(elem.span(), self.source, ErrorKind::InvalidRepr, "Expected a 2 element array."))
			}
		}
		else
		{
			Ok(None)
		}
	}

	fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Error>
		where V: de::DeserializeSeed<'de>
	{
		let elem = &self.elements[self.idx];
		if let Some(array) = elem.as_array()
		{
			if array.len() == 2
			{
				let ret = seed.deserialize(Deserializer::new(&array[1], self.source));
				self.idx += 1;
				ret
			}
			else
			{
				Err(Error::from_span(elem.span(), self.source, ErrorKind::InvalidRepr, "Expected a 2 element array."))
			}
		}
		else
		{
			Err(Error::from_span(elem.span(), self.source, ErrorKind::InvalidRepr, "Expected a 2 element array."))
		}
	}
}

struct MapHelper<'de, 'src: 'de>
{
	iter: btree_map::Iter<'de, String, ConfigElement>,
	value: Option<&'de ConfigElement>,
	source: Option<&'de Source<'src>>,
}

impl<'de, 'src> MapHelper<'de, 'src>
{
	fn new(elements: &'de BTreeMap<String, ConfigElement>, source: Option<&'de Source<'src>>) -> Self
	{
		Self {
			iter: elements.iter(),
			value: None,
			source: source,
		}
	}
}

impl<'de, 'src> de::MapAccess<'de> for MapHelper<'de, 'src>
{
	type Error = Error;

	fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Error>
		where K: de::DeserializeSeed<'de>
	{
		let next = self.iter.next();
		if let Some((k, v)) = next
		{
			self.value = Some(v);
			seed.deserialize(HackStringDeserializer::new(&*k)).map(Some)
		}
		else
		{
			Ok(None)
		}
	}

	fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Error>
		where V: de::DeserializeSeed<'de>
	{
		let v = self.value.unwrap();
		seed.deserialize(Deserializer::new(v, self.source))
	}
}

struct HackStringDeserializer<'de>
{
	string: &'de str,
}

impl<'de> HackStringDeserializer<'de>
{
	fn new(string: &'de str) -> Self
	{
		Self { string: string }
	}
}

struct VariantHelper<'de, 'src: 'de>
{
	element: Option<&'de ConfigElement>,
	source: Option<&'de Source<'src>>,
	span: Span,
}

impl<'de, 'src> VariantHelper<'de, 'src>
{
	fn new(element: Option<&'de ConfigElement>, source: Option<&'de Source<'src>>, span: Span) -> Self
	{
		Self {
			element: element,
			source: source,
			span: span,
		}
	}
}

impl<'de, 'src> de::VariantAccess<'de> for VariantHelper<'de, 'src>
{
	type Error = Error;
	fn unit_variant(self) -> Result<(), Error>
	{
		if self.element.is_some()
		{
			Err(Error::from_span(self.span, self.source, ErrorKind::InvalidRepr, "Expected a value."))
		}
		else
		{
			Ok(())
		}
	}

	fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Error>
		where T: de::DeserializeSeed<'de>
	{
		if let Some(elem) = self.element
		{
			seed.deserialize(Deserializer::new(elem, self.source))
		}
		else
		{
			Err(Error::from_span(self.span, self.source, ErrorKind::InvalidRepr, "Expected a table with a single element."))
		}
	}

	fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Error>
		where V: Visitor<'de>
	{
		if let Some(elem) = self.element
		{
			if let Some(array) = elem.as_array()
			{
				visitor.visit_seq(SeqHelper::new(array, self.source))
			}
			else
			{
				Err(Error::from_span(self.span, self.source, ErrorKind::InvalidRepr, "Expected a table with a single array element."))
			}
		}
		else
		{
			Err(Error::from_span(self.span, self.source, ErrorKind::InvalidRepr, "Expected a table with a single array element."))
		}
	}

	fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value, Error>
		where V: Visitor<'de>
	{
		if let Some(elem) = self.element
		{
			if let Some(table) = elem.as_table()
			{
				visitor.visit_map(MapHelper::new(table, self.source))
			}
			else
			{
				Err(Error::from_span(self.span, self.source, ErrorKind::InvalidRepr, "Expected a table with a single table element."))
			}
		}
		else
		{
			Err(Error::from_span(self.span, self.source, ErrorKind::InvalidRepr, "Expected a table with a single table element."))
		}
	}
}

/*
 * This is a hack because we do not support anything but string deserialization.
 * As far as I understand it we use it currently to deserialize enum variant
 * names as well as any static mapping (structs etc). One thing that this ends
 * up blocking is the table syntax for general mappings (since they might have
 * non-string keys).
 */
impl<'de> de::Deserializer<'de> for HackStringDeserializer<'de>
{
	type Error = Error;

	fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
		where V: Visitor<'de>
	{
		visitor.visit_borrowed_str(self.string)
	}

	forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
        byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

#[derive(Copy, Clone)]
struct Deserializer<'de, 'src: 'de>
{
	element: &'de ConfigElement,
	source: Option<&'de Source<'src>>,
}

impl<'de, 'src> Deserializer<'de, 'src>
{
	fn new(element: &'de ConfigElement, source: Option<&'de Source<'src>>) -> Self
	{
		Self { element: element, source: source }
	}

	fn error(&self, text: &str) -> Error
	{
		Error::from_span(self.element.span(), self.source, ErrorKind::InvalidRepr, text)
	}

	fn primitive<T: FromStr>(&self, name: &str) -> Result<T, Error>
		where T::Err: error::Error
	{
		if let Some(value) = self.element.as_value()
		{
			use std::error::Error;
			<T as FromStr>::from_str(value).map_err(|e| self.error(e.description()))
		}
		else
		{
			Err(self.error(&format!("Can't parse array/table as {}.", name)))
		}
	}
}

impl<'de, 'src> de::EnumAccess<'de> for Deserializer<'de, 'src>
{
	type Error = Error;
	type Variant = VariantHelper<'de, 'src>;

	fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Error>
		where V: de::DeserializeSeed<'de>
	{
		let span = self.element.span();
		match *self.element.kind()
		{
			ConfigElementKind::Value(_) => Ok((seed.deserialize(self)?, VariantHelper::new(None, self.source, span))),
			ConfigElementKind::Table(ref table) =>
			{
				let mut iter = table.iter();
				let ret = if let Some((k, v)) = iter.next()
				{
					Ok((seed.deserialize(HackStringDeserializer::new(&*k))?, VariantHelper::new(Some(v), self.source, span)))
				}
				else
				{
					Err(self.error(&format!("Table needs to have exactly one element.")))
				};
				if ret.is_ok() && iter.next().is_some()
				{
					Err(self.error(&format!("Table needs to have exactly one element.")))
				}
				else
				{
					ret
				}
			}
			_ => Err(self.error(&format!("Expected value or table."))),
		}
	}
}

impl<'de, 'src> de::Deserializer<'de> for Deserializer<'de, 'src>
{
	type Error = Error;

	fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Error>
		where V: Visitor<'de>
	{
		Err(self.error("deserialize_any unimplemented"))
	}

	fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Error>
		where V: Visitor<'de>
	{
		visitor.visit_bool(self.primitive("bool")?)
	}

	fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Error>
		where V: Visitor<'de>
	{
		visitor.visit_i8(self.primitive("i8")?)
	}

	fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Error>
		where V: Visitor<'de>
	{
		visitor.visit_i16(self.primitive("i16")?)
	}

	fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Error>
		where V: Visitor<'de>
	{
		visitor.visit_i32(self.primitive("i32")?)
	}

	fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Error>
		where V: Visitor<'de>
	{
		visitor.visit_i64(self.primitive("i64")?)
	}

	fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Error>
		where V: Visitor<'de>
	{
		visitor.visit_u8(self.primitive("u8")?)
	}

	fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Error>
		where V: Visitor<'de>
	{
		visitor.visit_u16(self.primitive("u16")?)
	}

	fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Error>
		where V: Visitor<'de>
	{
		visitor.visit_u32(self.primitive("u32")?)
	}

	fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Error>
		where V: Visitor<'de>
	{
		visitor.visit_u64(self.primitive("u64")?)
	}

	fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Error>
		where V: Visitor<'de>
	{
		visitor.visit_f32(self.primitive("f32")?)
	}

	fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Error>
		where V: Visitor<'de>
	{
		visitor.visit_f64(self.primitive("f64")?)
	}

	fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Error>
		where V: Visitor<'de>
	{
		if let Some(value) = self.element.as_value()
		{
			let mut chars = value.chars();
			let ret = visitor.visit_char(chars.next().unwrap());
			if chars.next().is_some()
			{
				Err(self.error(&format!("Can't parse '{}' a char.", value)))
			}
			else
			{
				ret
			}
		}
		else
		{
			Err(self.error(&format!("Can't parse array/table as char.")))
		}
	}

	fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Error>
		where V: Visitor<'de>
	{
		if let Some(value) = self.element.as_value()
		{
			visitor.visit_borrowed_str(value)
		}
		else
		{
			Err(self.error(&format!("Can't parse array/table as a string.")))
		}
	}

	fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Error>
		where V: Visitor<'de>
	{
		self.deserialize_str(visitor)
	}

	fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Error>
		where V: Visitor<'de>
	{
		if let Some(array) = self.element.as_array()
		{
			let mut bytes = vec![];

			for element in array
			{
				bytes.push(from_element(element, self.source)?);
			}

			visitor.visit_bytes(&bytes)
		}
		else
		{
			Err(self.error(&format!("Can't parse value/table as byte array.")))
		}
	}

	fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Error>
		where V: Visitor<'de>
	{
		if let Some(array) = self.element.as_array()
		{
			let mut bytes = vec![];

			for element in array
			{
				bytes.push(from_element(element, self.source)?);
			}

			visitor.visit_byte_buf(bytes)
		}
		else
		{
			Err(self.error(&format!("Can't parse value/table as byte array.")))
		}
	}

	fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Error>
		where V: Visitor<'de>
	{
		if let Some(value) = self.element.as_value()
		{
			if value.is_empty()
			{
				visitor.visit_none()
			}
			else
			{
				visitor.visit_some(self)
			}
		}
		else
		{
			visitor.visit_some(self)
		}
	}

	fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Error>
		where V: Visitor<'de>
	{
		if let Some(value) = self.element.as_value()
		{
			if value.is_empty()
			{
				visitor.visit_unit()
			}
			else
			{
				Err(self.error(&format!("Expected an empty value.")))
			}
		}
		else
		{
			Err(self.error(&format!("Expected an empty value.")))
		}
	}

	fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value, Error>
		where V: Visitor<'de>
	{
		self.deserialize_unit(visitor)
	}

	fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value, Error>
		where V: Visitor<'de>
	{
		visitor.visit_newtype_struct(self)
	}

	fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Error>
		where V: Visitor<'de>
	{
		if let Some(array) = self.element.as_array()
		{
			visitor.visit_seq(SeqHelper::new(array, self.source))
		}
		else
		{
			Err(self.error(&format!("Expected an array.")))
		}
	}

	fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Error>
		where V: Visitor<'de>
	{
		if let Some(array) = self.element.as_array()
		{
			if array.len() == len
			{
				visitor.visit_seq(SeqHelper::new(array, self.source))
			}
			else
			{
				Err(self.error(&format!("Expected an array with {} elements.", len)))
			}
		}
		else
		{
			Err(self.error(&format!("Expected an array.")))
		}
	}

	fn deserialize_tuple_struct<V>(self, _name: &'static str, len: usize, visitor: V) -> Result<V::Value, Error>
		where V: Visitor<'de>
	{
		self.deserialize_tuple(len, visitor)
	}

	fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Error>
		where V: Visitor<'de>
	{
		if let Some(array) = self.element.as_array()
		{
			visitor.visit_map(SeqHelper::new(array, self.source))
		}
		else
		{
			Err(self.error(&format!("Expected an array.")))
		}
	}

	fn deserialize_struct<V>(self, _name: &'static str, _fields: &'static [&'static str], visitor: V) -> Result<V::Value, Error>
		where V: Visitor<'de>
	{
		if let Some(table) = self.element.as_table()
		{
			visitor.visit_map(MapHelper::new(table, self.source))
		}
		else
		{
			Err(self.error(&format!("Expected a table.")))
		}
	}

	fn deserialize_enum<V>(self, _name: &'static str, _variants: &'static [&'static str], visitor: V) -> Result<V::Value, Error>
		where V: Visitor<'de>
	{
		visitor.visit_enum(self)
	}

	fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Error>
		where V: Visitor<'de>
	{
		self.deserialize_str(visitor)
	}

	fn deserialize_ignored_any<V>(self, _visitor: V) -> Result<V::Value, Error>
		where V: Visitor<'de>
	{
		Err(self.error("deserialize_ignored_any unimplemented"))
	}
}
