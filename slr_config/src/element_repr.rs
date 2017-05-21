use config_element::{Array, ConfigElement, Table, Value};
use slr_parser::{Error, ErrorKind, Source};
use std::collections::HashMap;
use std::default::Default;
use std::hash::Hash;
use std::str::FromStr;

/// Describes a way to convert a type to a ConfigElement and back.
pub trait ElementRepr
{
	/// Updates the contents of `self` based on values in the element.
	fn from_element<'l>(&mut self, elem: &ConfigElement, src: Option<&Source<'l>>) -> Result<(), Vec<Error>>;
	/// Creates an element that represents the contents of `self`.
	fn to_element(&self) -> ConfigElement;
}

macro_rules! element_repr_tuple_impl
{
	($($v: ident : $t: ident),*) =>
	{
		impl<$($t : $crate::ElementRepr + Default),*> $crate::ElementRepr for ($($t),*,)
		{
			fn from_element<'l>(&mut self, elem: &$crate::ConfigElement, src: Option<&$crate::Source<'l>>) -> Result<(), Vec<$crate::Error>>
			{
				match *elem.kind()
				{
					Array(ref arr) =>
					{
						let mut errors = vec![];
						let mut arr_itr = arr.iter();

						$(
							let mut $v: $t = Default::default();
						)*

						$(
							match arr_itr.next()
							{
								Some(ref val_elem) =>
								{
									$v.from_element(val_elem, src).map_err(|new_errors| errors.extend(new_errors)).ok();
								},
								None =>
								{
									return Err(vec![$crate::Error::from_span(elem.span(), src, $crate::ErrorKind::InvalidRepr, "Insufficient elements for a tuple")])
								}
							}
						)*

						if arr_itr.next().is_some()
						{
							return Err(vec![$crate::Error::from_span(elem.span(), src, $crate::ErrorKind::InvalidRepr, "Too many elements for a tuple")])
						}

						*self = ($($v),*,);

						if errors.is_empty()
						{
							Ok(())
						}
						else
						{
							Err(errors)
						}
					}
					Table(_) => Err(vec![$crate::Error::from_span(elem.span(), src, $crate::ErrorKind::InvalidRepr, "Cannot parse a table as a tuple")]),
					Value(_) => Err(vec![$crate::Error::from_span(elem.span(), src, $crate::ErrorKind::InvalidRepr, "Cannot parse a value as a tuple")]),
				}
			}

			fn to_element(&self) -> $crate::ConfigElement
			{
				let mut ret = ConfigElement::new_array();
				{
					let arr = ret.as_array_mut().unwrap();
					let ($(ref $v),*,) = *self;

					$(
						arr.push($v.to_element());
					)*
				}
				ret
			}
		}
	}
}

element_repr_tuple_impl!(a: A);
element_repr_tuple_impl!(a: A, b: B);
element_repr_tuple_impl!(a: A, b: B, c: C);
element_repr_tuple_impl!(a: A, b: B, c: C, d: D);
element_repr_tuple_impl!(a: A, b: B, c: C, d: D, e: E);
element_repr_tuple_impl!(a: A, b: B, c: C, d: D, e: E, f: F);
element_repr_tuple_impl!(a: A, b: B, c: C, d: D, e: E, f: F, g: G);
element_repr_tuple_impl!(a: A, b: B, c: C, d: D, e: E, f: F, g: G, h: H);

macro_rules! element_repr_via_str_impl
{
	($t: ty) =>
	{
		impl $crate::ElementRepr for $t
		{
			fn from_element<'l>(&mut self, elem: &$crate::ConfigElement, src: Option<&$crate::Source<'l>>) -> Result<(), Vec<$crate::Error>>
			{
				match *elem.kind()
				{
					$crate::Value(ref val) =>
					{
						match <$t as FromStr>::from_str(&val)
						{
							Ok(v) =>
							{
								*self = v;
								Ok(())
							}
							Err(_) => Err(vec![$crate::Error::from_span(elem.span(), src, $crate::ErrorKind::InvalidRepr, &format!("Cannot parse '{}' as {}", val, stringify!($t)))])
						}
					},
					$crate::Table(_) => Err(vec![$crate::Error::from_span(elem.span(), src, $crate::ErrorKind::InvalidRepr, &format!("Cannot parse a table as {}", stringify!($t)))]),
					$crate::Array(_) => Err(vec![$crate::Error::from_span(elem.span(), src, $crate::ErrorKind::InvalidRepr, &format!("Cannot parse an array as {}", stringify!($t)))]),
				}
			}

			fn to_element(&self) -> ConfigElement
			{
				ConfigElement::new_value(self.to_string())
			}
		}
	}
}

element_repr_via_str_impl!(i8);
element_repr_via_str_impl!(i16);
element_repr_via_str_impl!(i32);
element_repr_via_str_impl!(isize);
element_repr_via_str_impl!(u8);
element_repr_via_str_impl!(u16);
element_repr_via_str_impl!(u32);
element_repr_via_str_impl!(usize);
element_repr_via_str_impl!(f32);
element_repr_via_str_impl!(f64);
element_repr_via_str_impl!(String);
element_repr_via_str_impl!(bool);

impl<T: ElementRepr + Default> ElementRepr for Vec<T>
{
	fn from_element<'l>(&mut self, elem: &ConfigElement, src: Option<&Source<'l>>) -> Result<(), Vec<Error>>
	{
		match *elem.kind()
		{
			Array(ref arr) =>
			{
				let mut errors = vec![];
				self.clear();
				self.reserve(arr.len());
				for val_elem in arr
				{
					let mut val: T = Default::default();
					val.from_element(val_elem, src).map_err(|new_errors| errors.extend(new_errors)).ok();
					self.push(val);
				}
				if errors.is_empty()
				{
					Ok(())
				}
				else
				{
					Err(errors)
				}
			}
			Table(_) => Err(vec![Error::from_span(elem.span(), src, ErrorKind::InvalidRepr, "Cannot parse a table as 'Vec<T>'")]),
			Value(_) => Err(vec![Error::from_span(elem.span(), src, ErrorKind::InvalidRepr, "Cannot parse a value as 'Vec<T>'")]),
		}
	}

	fn to_element(&self) -> ConfigElement
	{
		let mut ret = ConfigElement::new_array();
		{
			let arr = ret.as_array_mut().unwrap();
			arr.reserve(self.len());
			for v in self
			{
				arr.push(v.to_element());
			}
		}
		ret
	}
}

impl<K: Eq + Hash + ToString + FromStr + Default, V: ElementRepr + Default> ElementRepr for HashMap<K, V>
    where K::Err: ToString
{
	fn from_element<'l>(&mut self, elem: &ConfigElement, src: Option<&Source<'l>>) -> Result<(), Vec<Error>>
	{
		match *elem.kind()
		{
			Table(ref map) =>
			{
				let mut errors = vec![];
				self.clear();

				for (k, v) in map
				{
					let key: K = k.parse()
						.map_err(|err: K::Err| {
							let err = err.to_string();
							errors.push(Error::from_span(elem.span(),
							                                   src,
							                                   ErrorKind::InvalidRepr,
							                                   &format!("Cannot parse '{}' as 'K': {}", k, err)));
						})
						.unwrap_or_default();
					let mut val: V = Default::default();

					val.from_element(v, src).map_err(|new_errors| errors.extend(new_errors)).ok();

					self.insert(key, val);
				}
				if errors.is_empty()
				{
					Ok(())
				}
				else
				{
					Err(errors)
				}
			}
			Array(_) =>
			{
				Err(vec![Error::from_span(elem.span(), src, ErrorKind::InvalidRepr, "Cannot parse an array as 'HashMap<K, V>'")])
			}
			Value(_) =>
			{
				Err(vec![Error::from_span(elem.span(), src, ErrorKind::InvalidRepr, "Cannot parse a value as 'HashMap<K, V>'")])
			}
		}
	}

	fn to_element(&self) -> ConfigElement
	{
		let mut ret = ConfigElement::new_table();
		{
			let tab = ret.as_table_mut().unwrap();
			for (k, v) in self
			{
				tab.insert(k.to_string(), v.to_element());
			}
		}
		ret
	}
}

#[macro_export]
#[doc(hidden)]
macro_rules! slr_def_struct_impl
{
	(
		struct $name: ident
		{
			$($field_name: ident : $field_type: ty = $field_init: expr),*
		}
	) =>
	{
		impl $crate::ElementRepr for $name
		{
			fn from_element<'l>(&mut self, elem: &$crate::ConfigElement, src: Option<&$crate::Source<'l>>) -> Result<(), Vec<$crate::Error>>
			{
				match *elem.kind()
				{
					$crate::Table(ref table) =>
					{
						let mut errors = vec![];
						for (k, v) in table
						{
							match &k[..]
							{
								$(
									stringify!($field_name) =>
									{
										// Use UFCS for a better error message.
										<$field_type as $crate::ElementRepr>::from_element(&mut self.$field_name, v, src).map_err(|new_errors| errors.extend(new_errors)).ok();
									},
								)*
								_ => errors.push($crate::Error::from_span(elem.span(), src, $crate::ErrorKind::UnknownField,
									&format!("{} does not have a field named {}", stringify!($name), k)))
							}
						}
						if errors.is_empty()
						{
							Ok(())
						}
						else
						{
							Err(errors)
						}
					},
					$crate::Value(_) => Err(vec![$crate::Error::from_span(elem.span(), src, $crate::ErrorKind::InvalidRepr, &format!("Cannot parse a value as {}", stringify!($name)))]),
					$crate::Array(_) => Err(vec![$crate::Error::from_span(elem.span(), src, $crate::ErrorKind::InvalidRepr, &format!("Cannot parse an array as {}", stringify!($name)))]),
				}
			}

			fn to_element(&self) -> $crate::ConfigElement
			{
				let mut ret = $crate::ConfigElement::new_table();
				{
					let tab = ret.as_table_mut().unwrap();
					$(
						tab.insert(stringify!($field_name).to_string(), <$field_type as $crate::ElementRepr>::to_element(&self.$field_name));
					)*
				}
				ret
			}
		}
	}
}

#[macro_export]
#[doc(hidden)]
macro_rules! slr_def_enum_impl
{
	(
		enum $name: ident
		{
			$($var_name: ident),*
		}
	) =>
	{
		impl $crate::ElementRepr for $name
		{
			fn from_element<'l>(&mut self, elem: &$crate::ConfigElement, src: Option<&$crate::Source<'l>>) -> Result<(), Vec<$crate::Error>>
			{
				match *elem.kind()
				{
					$crate::Value(ref val) =>
					{
						match &val[..]
						{
							$(
								stringify!($var_name) =>
								{
									*self = $name::$var_name;
									Ok(())
								}
								,
							)*
							_ => Err(vec![$crate::Error::from_span(elem.span(), src, $crate::ErrorKind::InvalidRepr, &format!("{} has no variant named '{}'", stringify!($name), val))])
						}
					},
					$crate::Table(_) => Err(vec![$crate::Error::from_span(elem.span(), src, $crate::ErrorKind::InvalidRepr, &format!("Cannot parse a table as {}", stringify!($name)))]),
					$crate::Array(_) => Err(vec![$crate::Error::from_span(elem.span(), src, $crate::ErrorKind::InvalidRepr, &format!("Cannot parse an array as {}", stringify!($name)))]),
				}
			}

			fn to_element(&self) -> $crate::ConfigElement
			{
				match *self
				{
					$(
						$name::$var_name => $crate::ConfigElement::new_value(stringify!($var_name).to_string()),
					)*
				}
			}
		}
	}
}

/** A macro to define the compile-time schemas for configuration elements.
You can use this macro to define structs and enums, like so:

~~~
#[macro_use]
extern crate slr_config;

slr_def!
{
	#[derive(Clone)] // Attributes supported.
	pub struct TestSchema
	{
		pub key: u32 = 0, // The rhs assignments are initializer expressions.
		pub arr: Vec<u32> = vec![]
	}
}

slr_def!
{
	pub enum TestEnum
	{
		VariantA,
		VariantB
	}
}

# fn main() {}
~~~

The types declared by both invocations implement `ElementRepr`, and the struct
versions also implement a `new` constructor which creates the type with the
default value specified by the macro invocation.
*/
#[macro_export]
macro_rules! slr_def
{
	(
		$(#[$attrs:meta])*
		pub struct $name: ident
		{
			$(pub $field_name: ident : $field_type: ty = $field_init: expr),* $(,)*
		}
	) =>
	{
		$(#[$attrs])*
		pub struct $name
		{
			$(pub $field_name : $field_type),*
		}

		impl $name
		{
			pub fn new() -> $name
			{
				$name
				{
					$($field_name: $field_init),*
				}
			}
		}

		slr_def_struct_impl!
		{
			struct $name
			{
				$($field_name : $field_type = $field_init),*
			}
		}
	};

	(
		$(#[$attrs:meta])*
		struct $name: ident
		{
			$($field_name: ident : $field_type: ty = $field_init: expr),* $(,)*
		}
	) =>
	{
		$(#[$attrs])*
		struct $name
		{
			$($field_name : $field_type),*
		}

		impl $name
		{
			fn new() -> $name
			{
				$name
				{
					$($field_name: $field_init),*
				}
			}
		}

		slr_def_struct_impl!
		{
			struct $name
			{
				$($field_name : $field_type = $field_init),*
			}
		}
	};

	(
		$(#[$attrs:meta])*
		enum $name: ident
		{
			$($var_name: ident),* $(,)*
		}
	) =>
	{
		$(#[$attrs])*
		enum $name
		{
			$($var_name),*
		}

		slr_def_enum_impl!
		{
			enum $name
			{
				$($var_name),*
			}
		}
	};

	(
		$(#[$attrs:meta])*
		pub enum $name: ident
		{
			$($var_name: ident),* $(,)*
		}
	) =>
	{
		$(#[$attrs])*
		pub enum $name
		{
			$($var_name),*
		}

		slr_def_enum_impl!
		{
			enum $name
			{
				$($var_name),*
			}
		}
	};
}
