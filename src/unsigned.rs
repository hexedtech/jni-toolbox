use jni::objects::JObject;
use crate::{FromJava, IntoJava, IntoJavaObject, from_java::FromJavaObject};

pub trait JavaTranslatable<'j> : Sized {
	type To : IntoJava<'j>;
	fn translate_to(self) -> Self::To;
	fn translate_back(val: Self::To) -> Result<Self, jni::errors::Error>;
}

macro_rules! translate {
	($from:ty, $to:ty, $self:ident, $cast_to:expr, $val:ident, $cast_back:expr) => {
		impl<'j> JavaTranslatable<'j> for $from {
			type To = $to;
			fn translate_to($self) -> Self::To {
				$cast_to
			}

			fn translate_back($val: Self::To) -> Result<Self, jni::errors::Error> {
				$cast_back
			}
		}

		impl<'j> JavaTranslatable<'j> for Option<$from> {
			type To = Option<$to>;
			fn translate_to(self) -> Self::To {
				match self {
					Some(val) => Some(val.translate_to()),
					None => None,
				}
			}

			fn translate_back(val: Self::To) -> Result<Self, jni::errors::Error> {
				match val {
					Some(val) => Ok(Some(<$from as JavaTranslatable>::translate_back(val)?)),
					None => Ok(None),
				}
			}
		}

		impl<'j> JavaTranslatable<'j> for Vec<$from> {
			type To = Vec<$to>;
			fn translate_to(self) -> Self::To {
				self.iter().map(|v| v.translate_to()).collect()
			}
			fn translate_back(val: Self::To) -> Result<Self, jni::errors::Error> {
				let mut res = Vec::new();
				for el in val {
					res.push(<$from as JavaTranslatable>::translate_back(el)?);
				}
				Ok(res)
			}
		}

		impl<'j> IntoJava<'j> for $from {
			type Ret = <<Self as JavaTranslatable<'j>>::To as IntoJava<'j>>::Ret;

			fn signature() -> (String, jni::signature::JavaType) {
				<Self as JavaTranslatable>::To::signature()
			}

			fn into_java(self, e: &mut jni::Env<'j>) -> Result<Self::Ret, jni::errors::Error> {
				self.translate_to().into_java(e)
			}

			fn into_jvalue(self, e: &mut jni::Env<'j>) -> Result<jni::JValueOwned<'j>, jni::errors::Error> {
				self.translate_to().into_jvalue(e)
			}
		}

		impl<'j> IntoJavaObject<'j> for Option<$from> {
			const CLASS: &'static str = <Self as JavaTranslatable>::To::CLASS;
			fn into_java_object(self, env: &mut jni::Env<'j>) -> Result<JObject<'j>, jni::errors::Error> {
				self.translate_to().into_java_object(env)	
			}
		}

		impl<'j> IntoJavaObject<'j> for Vec<$from> {
			const CLASS: &'static str = <Self as JavaTranslatable>::To::CLASS;
			const ARRAY_DEPTH: usize = <Self as JavaTranslatable>::To::ARRAY_DEPTH;
			fn into_java_object(self, env: &mut jni::Env<'j>) -> Result<JObject<'j>, jni::errors::Error> {
				self.translate_to().into_java_object(env)
			}
		}

		impl<'j> FromJava<'j> for $from {
			type From = <<Self as JavaTranslatable<'j>>::To as FromJava<'j>>::From;

			#[inline]
			fn from_java(env: &mut jni::Env<'j>, value: Self::From) -> Result<Self, jni::errors::Error> {
				let val = <Self as JavaTranslatable<'j>>::To::from_java(env, value)?;
				Self::translate_back(val)
			}

			fn from_jvalue(env: &mut jni::Env<'j>, value: jni::JValueOwned<'j>) -> Result<Self, jni::errors::Error> {
				let val = <Self as JavaTranslatable<'j>>::To::from_jvalue(env, value)?;
				Self::translate_back(val)
			}
		}

		impl<'j> FromJavaObject<'j> for Option<$from> {
			fn from_java_object(obj: JObject<'j>, env: &mut jni::Env<'j>) -> Result<Self, jni::errors::Error> {
				Self::translate_back(<Self as JavaTranslatable<'j>>::To::from_java_object(obj, env)?)
			}
		}

		impl<'j> FromJava<'j> for Vec<$from> {
			type From = <<Self as JavaTranslatable<'j>>::To as FromJava<'j>>::From;

			fn from_java(env: &mut jni::Env<'j>, value: Self::From) -> Result<Self, jni::errors::Error> {
				let val = <Self as JavaTranslatable<'j>>::To::from_java(env, value)?;
				Self::translate_back(val)
			}

			fn from_jvalue(env: &mut jni::Env<'j>, value: jni::JValueOwned<'j>) -> Result<Self, jni::errors::Error> {
				let val = <Self as JavaTranslatable<'j>>::To::from_jvalue(env, value)?;
				Self::translate_back(val)
			}
		}
	};
}

// special "magic" impl: bytes are "usually" used raw
// in the future we'll come up with a way to allow both usages, but for now we need this
translate!(u8, i8, self, i8::from_ne_bytes(self.to_ne_bytes()), val, Ok(u8::from_ne_bytes(val.to_ne_bytes())));

// normal impls
translate!(u16, i32, self, self as i32, val, Self::try_from(val).map_err(|_| jni::errors::Error::WrongObjectType));
translate!(u32, i64, self, self as i64, val, Self::try_from(val).map_err(|_| jni::errors::Error::WrongObjectType));
