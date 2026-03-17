use jni::{objects::{JObject, JObjectArray, JPrimitiveArray, JString, TypeArray}, signature::RuntimeMethodSignature};

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

/// Specifies how a Java object can be converted into a Rust type.
pub trait FromJavaObject<'j> : Sized {
	/// Attempts to convert this Java object into a Rust type.
	fn from_java_object(obj: JObject<'j>, env: &mut jni::Env<'j>) -> Result<Self, jni::errors::Error>;
}

impl<'j, X: FromJavaObject<'j>> FromJava<'j> for X {
	type From = JObject<'j>;

	#[inline]
	fn from_java(env: &mut jni::Env<'j>, value: Self::From) -> Result<Self, jni::errors::Error> {
		Self::from_java_object(value, env)
	}

	fn from_jvalue(env: &mut jni::Env<'j>, value: jni::JValueOwned<'j>) -> Result<Self, jni::errors::Error> {
		Self::from_java_object(value.l()?, env)
	}
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
	($t: ty, $from_type: ty, $sig:literal, $ext:ident, $method:literal) => {
		impl<'j> FromJava<'j> for $t {
			type From = $from_type;
		
			#[inline]
			fn from_java(_: &mut jni::Env, value: Self::From) -> Result<Self, jni::errors::Error> {
				Ok(value)
			}

			fn from_jvalue(_: &mut jni::Env, value: jni::JValueOwned<'j>) -> Result<Self, jni::errors::Error> {
				value.$ext()
			}
		}

		impl<'j> FromJava<'j> for Vec<$t> {
			type From = JPrimitiveArray<'j, $t>;
	
			fn from_java(env: &mut jni::Env<'j>, value: Self::From) -> Result<Self, jni::errors::Error> {
				let len = value.len(env)?;
				let mut out = vec![<$t>::default(); len];
				value.get_region(env, 0, &mut out)?;
				Ok(out)
			}

			fn from_jvalue(env: &mut jni::Env<'j>, value: jni::JValueOwned<'j>) -> Result<Self, jni::errors::Error> {
				let jarr = env.cast_local::<JPrimitiveArray<'j, $t>>(value.l()?)?;
				Self::from_java(env, jarr)
			}
		}

		impl<'j> FromJavaObject<'j> for Option<$t> {
			fn from_java_object(obj: JObject<'j>, env: &mut jni::Env<'j>) -> Result<Self, jni::errors::Error> {
				if obj.is_null() {
					return Ok(None);
				}
				let sig = RuntimeMethodSignature::from_str(concat!("()", $sig))?;
				let jval = env.call_method(&obj, jni::jni_str!($method), sig.method_signature(), &[])?;
				Ok(Some(jval.$ext()?))
			}
		}

		impl<'j> FromJava<'j> for Option<Vec<$t>> {
			type From = JPrimitiveArray<'j, $t>;

			fn from_java(env: &mut jni::Env<'j>, value: Self::From) -> Result<Self, jni::errors::Error> {
				if value.is_null() {
					return Ok(None)
				}

				<Vec<$t>>::from_java(env, value).map(|res| Some(res))
			}

			fn from_jvalue(env: &mut jni::Env<'j>, value: jni::JValueOwned<'j>) -> Result<Self, jni::errors::Error> {
				let jarr = env.cast_local::<JPrimitiveArray<'j, $t>>(value.l()?)?;
				Self::from_java(env, jarr)
			}
		}
	};
}

auto_from_java!(i8, jni::sys::jbyte, "B", b, "byteValue");
auto_from_java!(i16, jni::sys::jshort, "S", s, "shortValue");
auto_from_java!(i32, jni::sys::jint, "I", i, "intValue");
auto_from_java!(i64, jni::sys::jlong, "J", j, "longValue");
auto_from_java!(f32, jni::sys::jfloat, "F", f, "floatValue");
auto_from_java!(f64, jni::sys::jdouble, "D", d, "doubleValue");
auto_from_java!(bool, jni::sys::jboolean, "Z", z, "booleanValue");

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
		jni::char_from_java(value).map_err(|e| jni::errors::Error::InvalidUtf16 { source: e })
	}

	fn from_jvalue(_: &mut jni::Env<'j>, value: jni::JValueOwned<'j>) -> Result<Self, jni::errors::Error> {
		jni::char_from_java(value.c()?)
			.map_err(|e| jni::errors::Error::InvalidUtf16 { source: e })
	}
}

impl<'j> FromJavaObject<'j> for String {
	fn from_java_object(obj: JObject<'j>, env: &mut jni::Env<'j>) -> Result<Self, jni::errors::Error> {
		let jstr = JString::cast_local(env, obj)?; 
		Ok(jstr.to_string())
	}
}

impl<'j, T: FromJavaObject<'j>> FromJavaObject<'j> for Option<T> {
	fn from_java_object(obj: JObject<'j>, env: &mut jni::Env<'j>) -> Result<Self, jni::errors::Error> {
		if obj.is_null() {
			Ok(None)
		} else {
			Ok(Some(T::from_java_object(obj, env)?))
		}
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
impl<'j> FromJavaObject<'j> for uuid::Uuid {
	fn from_java_object(obj: JObject<'j>, env: &mut jni::Env<'j>) -> Result<Self, jni::errors::Error> {
		let lsb = u64::from_ne_bytes(
			env.call_method(&obj, jni::jni_str!("getLeastSignificantBits"), jni::jni_sig!("()J"), &[])?
				.j()?
				.to_ne_bytes()
		);

		let msb = u64::from_ne_bytes(
			env.call_method(&obj, jni::jni_str!("getMostSignificantBits"), jni::jni_sig!("()J"), &[])?
				.j()?
				.to_ne_bytes()
		);
		
		Ok(uuid::Uuid::from_u64_pair(msb, lsb))
	}
}
