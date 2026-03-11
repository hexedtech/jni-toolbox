use jni::objects::{JObject, JObjectArray, JPrimitiveArray, JString, TypeArray};


/// Used in the generated code to have proper type bindings. You probably didn't want
/// to call this directly.
pub fn from_java_static<'j, T: FromJava<'j>>(env: &mut jni::Env<'j>, val: T::From) -> Result<T, jni::errors::Error> {
	T::from_java(env, val)
}

/// Used in the generated code to have proper type bindings. You probably didn't want
/// to call this directly.
pub fn from_jvalue_static<'j, T: FromJava<'j>>(env: &mut jni::Env<'j>, val: jni::JValueOwned<'j>) -> Result<T, jni::errors::Error> {
	T::from_jvalue(env, val)
}

/// Specifies how a Java type should be converted before being fed to Rust.
pub trait FromJava<'j> : Sized {
	/// The JNI type representing the input.
	type From : Sized;
	/// Attempts to convert this Java object into its Rust counterpart.
	fn from_java(env: &mut jni::Env<'j>, value: Self::From) -> Result<Self, jni::errors::Error>;
	/// Attempts to convert this Rust object into a JValue (used in constructors).
	fn from_jvalue(env: &mut jni::Env<'j>, value: jni::JValueOwned<'j>) -> Result<Self, jni::errors::Error>;
}

impl<'j> FromJava<'j> for JObject<'j> {
	type From = JObject<'j>;

	#[inline]
	fn from_java(_: &mut jni::Env, value: Self::From) -> Result<Self, jni::errors::Error> {
		Ok(value)
	}

	fn from_jvalue(_: &mut jni::Env, value: jni::JValueOwned<'j>) -> Result<Self, jni::errors::Error> {
		value.l()
	}
}

impl<'j> FromJava<'j> for JString<'j> {
	type From = JString<'j>;

	#[inline]
	fn from_java(_: &mut jni::Env, value: Self::From) -> Result<Self, jni::errors::Error> {
		Ok(value)
	}

	fn from_jvalue(env: &mut jni::Env, value: jni::JValueOwned<'j>) -> Result<Self, jni::errors::Error> {
		JString::cast_local(env, value.l()?)
	}
}

impl<'j> FromJava<'j> for JObjectArray<'j> {
	type From = JObjectArray<'j>;

	#[inline]
	fn from_java(_: &mut jni::Env, value: Self::From) -> Result<Self, jni::errors::Error> {
		Ok(value)
	}

	fn from_jvalue(env: &mut jni::Env, value: jni::JValueOwned<'j>) -> Result<Self, jni::errors::Error> {
		let val = value.l()?;
		Self::From::cast_local(env, val)
	}
}

macro_rules! auto_from_java {
	($t: ty, $j: ty, $ext:ident) => {
		impl<'j> FromJava<'j> for $t {
			type From = $j;
		
			#[inline]
			fn from_java(_: &mut jni::Env, value: Self::From) -> Result<Self, jni::errors::Error> {
				Ok(value)
			}

			fn from_jvalue(_: &mut jni::Env, value: jni::JValueOwned<'j>) -> Result<Self, jni::errors::Error> {
				value.$ext()
			}
		}
	};
}

auto_from_java!(i8, jni::sys::jbyte, b);
auto_from_java!(i16, jni::sys::jshort, s);
auto_from_java!(i32, jni::sys::jint, i);
auto_from_java!(i64, jni::sys::jlong, j);
auto_from_java!(f32, jni::sys::jfloat, f);
auto_from_java!(f64, jni::sys::jdouble, d);
auto_from_java!(bool, jni::sys::jboolean, z);

impl<'j> FromJava<'j> for u8 {
	type From = jni::sys::jbyte;

	#[inline]
	fn from_java(_: &mut jni::Env, value: Self::From) -> Result<Self, jni::errors::Error> {
		Ok(value as u8)
	}

	fn from_jvalue(_: &mut jni::Env, value: jni::JValueOwned<'j>) -> Result<Self, jni::errors::Error> {
		Ok(value.b()? as u8)
	}
}

impl<'j, T: TypeArray> FromJava<'j> for JPrimitiveArray<'j, T> {
	type From = JPrimitiveArray<'j, T>;

	#[inline]
	fn from_java(_: &mut jni::Env, value: Self::From) -> Result<Self, jni::errors::Error> {
		Ok(value)
	}

	fn from_jvalue(_env: &mut jni::Env<'j>, _value: jni::JValueOwned<'j>) -> Result<Self, jni::errors::Error> {
		todo!()
	}
}

impl<'j> FromJava<'j> for char {
	type From = jni::sys::jchar;

	#[inline]
	fn from_java(_: &mut jni::Env, value: Self::From) -> Result<Self, jni::errors::Error> {
		char::from_u32(value.into()).ok_or_else(|| jni::errors::Error::WrongJValueType("char", "invalid u16"))
	}

	fn from_jvalue(_: &mut jni::Env<'j>, value: jni::JValueOwned<'j>) -> Result<Self, jni::errors::Error> {
		value.c_char()
	}
}

impl<'j> FromJava<'j> for String {
	type From = JString<'j>;

	fn from_java(_: &mut jni::Env<'j>, value: Self::From) -> Result<Self, jni::errors::Error> {
		if value.is_null() { return Err(jni::errors::Error::NullPtr("string can't be null")) };
		Ok(value.to_string())
	}

	fn from_jvalue(env: &mut jni::Env<'j>, value: jni::JValueOwned<'j>) -> Result<Self, jni::errors::Error> {
		let jstr = JString::cast_local(env, value.l()?)?; 
		Self::from_java(env, jstr)
	}
}

impl<'j, T> FromJava<'j> for Option<T>
where
	T: FromJava<'j, From: AsRef<JObject<'j>>>,
{
	type From = T::From;

	fn from_java(env: &mut jni::Env<'j>, value: Self::From) -> Result<Self, jni::errors::Error> {
		if value.as_ref().is_null() { return Ok(None) };
		Ok(Some(T::from_java(env, value)?))
	}

	fn from_jvalue(env: &mut jni::Env<'j>, value: jni::JValueOwned<'j>) -> Result<Self, jni::errors::Error> {
		// TODO this is only for objects, right??
		if value.borrow().l()?.is_null() { return Ok(None) };
		Ok(Some(T::from_jvalue(env, value)?))
	}
}

impl<'j, T: FromJava<'j, From = JObject<'j>>> FromJava<'j> for Vec<T> {
	type From = JObjectArray<'j>;

	fn from_java(env: &mut jni::Env<'j>, value: Self::From) -> Result<Self, jni::errors::Error> {
		let len = value.len(env)?;
		let mut out = Vec::new();
		for i in 0..len {
			let el = value.get_element(env, i)?;
			out.push(T::from_java(env, el)?);
		}
		Ok(out)
	}

	fn from_jvalue(env: &mut jni::Env<'j>, value: jni::JValueOwned<'j>) -> Result<Self, jni::errors::Error> {
		let jarr: JObjectArray<'j, JObject<'j>> = env.cast_local::<JObjectArray>(value.l()?)?;
		Self::from_java(env, jarr)
	}
}

macro_rules! auto_from_java_primitive_array {
	($primitive:ty) => {
		impl<'j> FromJava<'j> for Vec<$primitive> {
			type From = JPrimitiveArray<'j, $primitive>;
		
			fn from_java(env: &mut jni::Env<'j>, value: Self::From) -> Result<Self, jni::errors::Error> {
				let len = value.len(env)?;
				let mut out = vec![<$primitive>::default(); len];
				value.get_region(env, 0, &mut out)?;
				Ok(out)
			}

			fn from_jvalue(env: &mut jni::Env<'j>, value: jni::JValueOwned<'j>) -> Result<Self, jni::errors::Error> {
				let jarr: JPrimitiveArray<'j, $primitive> = env.cast_local::<JPrimitiveArray<$primitive>>(value.l()?)?;
				Self::from_java(env, jarr)
			}
		}
	};
}

auto_from_java_primitive_array!(i8);
auto_from_java_primitive_array!(i16);
auto_from_java_primitive_array!(i32);
auto_from_java_primitive_array!(i64);
auto_from_java_primitive_array!(f32);
auto_from_java_primitive_array!(f64);
auto_from_java_primitive_array!(bool);

impl<'j> FromJava<'j> for Vec<u8> {
	type From = JPrimitiveArray<'j, i8>;

	fn from_java(env: &mut jni::Env<'j>, value: Self::From) -> Result<Self, jni::errors::Error> {
		let len = value.len(env)?;
		let mut out = vec![<i8>::default(); len];
		value.get_region(env, 0, &mut out)?;
		Ok(out.into_iter().map(|x| x as u8).collect())
	}

	fn from_jvalue(env: &mut jni::Env<'j>, value: jni::JValueOwned<'j>) -> Result<Self, jni::errors::Error> {
		let jarr: JPrimitiveArray<'j, i8> = env.cast_local::<JPrimitiveArray<i8>>(value.l()?)?;
		Self::from_java(env, jarr)
	}
}

impl<'j> FromJava<'j> for Vec<char> {
	type From = JPrimitiveArray<'j, u16>;

	fn from_java(env: &mut jni::Env<'j>, value: Self::From) -> Result<Self, jni::errors::Error> {
		let len = value.len(env)?;
		let mut out = vec![<u16>::default(); len];
		value.get_region(env, 0, &mut out)?;
		Ok(
			out
				.into_iter()
				.map(|x| char::from_u32(x.into()).unwrap_or_default())
				.collect()
		)
	}

	fn from_jvalue(env: &mut jni::Env<'j>, value: jni::JValueOwned<'j>) -> Result<Self, jni::errors::Error> {
		let jarr = env.cast_local::<JPrimitiveArray<u16>>(value.l()?)?;
		Self::from_java(env, jarr)
	}
}

#[cfg(feature = "uuid")]
impl<'j> FromJava<'j> for uuid::Uuid {
	type From = JObject<'j>;
	fn from_java(env: &mut jni::Env<'j>, uuid: Self::From) -> Result<Self, jni::errors::Error> {
		let lsb = u64::from_ne_bytes(
			env.call_method(&uuid, jni::jni_str!("getLeastSignificantBits"), jni::jni_sig!("()J"), &[])?
				.j()?
				.to_ne_bytes()
		);

		let msb = u64::from_ne_bytes(
			env.call_method(&uuid, jni::jni_str!("getMostSignificantBits"), jni::jni_sig!("()J"), &[])?
				.j()?
				.to_ne_bytes()
		);
		
		Ok(uuid::Uuid::from_u64_pair(msb, lsb))
	}

	fn from_jvalue(env: &mut jni::Env<'j>, value: jni::JValueOwned<'j>) -> Result<Self, jni::errors::Error> {
		Self::from_java(env, value.l()?)
	}
}
