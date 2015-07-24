use config_element::{ConfigElement, Value, Table, Array};
use lex::{Error, Span, Source};
use std::str::FromStr;

fn make_error<'l>(msg: &str, span: Span, src: Option<&Source<'l>>) -> Result<(), Error>
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
