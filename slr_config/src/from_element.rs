use config_element::{ConfigElement, Value, Table, Array};
use lex::{Error, Span, Source};
use std::str::FromStr;
use std::default::Default;

pub fn make_error<'l>(msg: &str, span: Span, src: Option<&Source<'l>>) -> Result<(), Error>
{
	match src
	{
		Some(src) => Error::from_span(src, span, msg),
		None => Err(Error::new(msg.to_string()))
	}
}

pub trait FromElement<'l>
{
	fn from_element(&mut self, elem: &ConfigElement, src: Option<&Source<'l>>) -> Result<(), Error>;
}

macro_rules! impl_from_element
{
	($t: ty) =>
	{
		impl<'l> FromElement<'l> for $t
		{
			fn from_element(&mut self, elem: &ConfigElement, src: Option<&Source<'l>>) -> Result<(), Error>
			{
				match *elem.kind()
				{
					Value(ref val) =>
					{
						match <$t as FromStr>::from_str(&val)
						{
							Ok(v) =>
							{
								*self = v;
								Ok(())
							}
							Err(_) => make_error(&format!("Cannot parse '{}' as {}", val, stringify!($t)), elem.span(), src)
						}
					},
					Table(_) => make_error(&format!("Cannot parse a table as {}", stringify!($t)), elem.span(), src),
					Array(_) => make_error(&format!("Cannot parse an array as {}", stringify!($t)), elem.span(), src),
				}
			}
		}
	}
}

impl_from_element!(i8);
impl_from_element!(i16);
impl_from_element!(i32);
impl_from_element!(isize);
impl_from_element!(u8);
impl_from_element!(u16);
impl_from_element!(u32);
impl_from_element!(usize);
impl_from_element!(f32);
impl_from_element!(f64);
impl_from_element!(String);

impl<'l, T: FromElement<'l> + Default> FromElement<'l> for Vec<T>
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
		impl<'l> FromElement<'l> for $name
		{
			fn from_element(&mut self, elem: &ConfigElement, src: Option<&Source<'l>>) -> Result<(), Error>
			{
				match *elem.kind()
				{
					Table(ref table) =>
					{
						$(
							match table.get(stringify!($field_name))
							{
								Some(v) =>
								{
									// Use UFCS for a better error message.
									try!(<$field_type as FromElement>::from_element(&mut self.$field_name, v, src))
								},
								_ => (),
							}
						)*
						Ok(())
					},
					Value(_) => make_error(&format!("Cannot parse a value as {}", stringify!($name)), elem.span(), src),
					Array(_) => make_error(&format!("Cannot parse an array as {}", stringify!($name)), elem.span(), src),
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
	}
}
