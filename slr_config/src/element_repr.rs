use config_element::{ConfigElement, Value, Table, Array};
use lex::{Error, Span, Source};
use std::str::FromStr;
use std::default::Default;

// TODO: Remove this.
pub fn make_error<'l>(msg: &str, span: Span, src: Option<&Source<'l>>) -> Result<(), Error>
{
	match src
	{
		Some(src) => Error::from_span(src, span, msg),
		None => Err(Error::new(msg.to_string()))
	}
}

/// Describes a way to convert a type to a ConfigElement and back.
pub trait ElementRepr<'l>
{
	/// Updates the contents of `self` based on values in the element.
	fn from_element(&mut self, elem: &ConfigElement, src: Option<&Source<'l>>) -> Result<(), Error>;
	/// Creates an element that represents the contents of `self`.
	fn to_element(&self) -> ConfigElement;
}

macro_rules! element_repr_via_str_impl
{
	($t: ty) =>
	{
		impl<'l> $crate::ElementRepr<'l> for $t
		{
			fn from_element(&mut self, elem: &$crate::ConfigElement, src: Option<&$crate::Source<'l>>) -> Result<(), $crate::Error>
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
							Err(_) => $crate::make_error(&format!("Cannot parse '{}' as {}", val, stringify!($t)), elem.span(), src)
						}
					},
					$crate::Table(_) => $crate::make_error(&format!("Cannot parse a table as {}", stringify!($t)), elem.span(), src),
					$crate::Array(_) => $crate::make_error(&format!("Cannot parse an array as {}", stringify!($t)), elem.span(), src),
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

impl<'l, T: ElementRepr<'l> + Default> ElementRepr<'l> for Vec<T>
{
	fn from_element(&mut self, elem: &ConfigElement, src: Option<&Source<'l>>) -> Result<(), Error>
	{
		match *elem.kind()
		{
			Array(ref arr) =>
			{
				self.clear();
				self.reserve(arr.len());
				for val_elem in arr
				{
					let mut val: T = Default::default();
					try!(val.from_element(val_elem, src));
					self.push(val);
				}
				Ok(())
			},
			Table(_) => make_error("Cannot parse a table as 'Vec<T>'", elem.span(), src),
			Value(_) => make_error("Cannot parse a value as 'Vec<T>'", elem.span(), src),
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

#[macro_export]
macro_rules! slr_def_struct_impl
{
	(
		struct $name: ident
		{
			$($field_name: ident : $field_type: ty = $field_init: expr),*
		}
	) =>
	{
		impl<'l> $crate::ElementRepr<'l> for $name
		{
			fn from_element(&mut self, elem: &$crate::ConfigElement, src: Option<&$crate::Source<'l>>) -> Result<(), $crate::Error>
			{
				match *elem.kind()
				{
					$crate::Table(ref table) =>
					{
						$(
							match table.get(stringify!($field_name))
							{
								Some(v) =>
								{
									// Use UFCS for a better error message.
									try!(<$field_type as $crate::ElementRepr>::from_element(&mut self.$field_name, v, src))
								},
								_ => (),
							}
						)*
						Ok(())
					},
					$crate::Value(_) => $crate::make_error(&format!("Cannot parse a value as {}", stringify!($name)), elem.span(), src),
					$crate::Array(_) => $crate::make_error(&format!("Cannot parse an array as {}", stringify!($name)), elem.span(), src),
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
macro_rules! slr_def_enum_impl
{
	(
		enum $name: ident
		{
			$($var_name: ident),*
		}
	) =>
	{
		impl<'l> $crate::ElementRepr<'l> for $name
		{
			fn from_element(&mut self, elem: &$crate::ConfigElement, src: Option<&$crate::Source<'l>>) -> Result<(), $crate::Error>
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
							_ => $crate::make_error(&format!("{} has no variant named '{}'", stringify!($name), val), elem.span(), src)
						}
					},
					$crate::Table(_) => $crate::make_error(&format!("Cannot parse a table as {}", stringify!($name)), elem.span(), src),
					$crate::Array(_) => $crate::make_error(&format!("Cannot parse an array as {}", stringify!($name)), elem.span(), src),
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

#[macro_export]
macro_rules! slr_def
{
	(
		$(#[$attrs:meta])*
		pub struct $name: ident
		{
			$($field_name: ident : $field_type: ty = $field_init: expr),*
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
			$($field_name: ident : $field_type: ty = $field_init: expr),*
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
			$($var_name: ident),*
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
			$($var_name: ident),*
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
