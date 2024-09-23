use jni::objects::{JObject, JObjectArray, JPrimitiveArray, JString, TypeArray};


/// Used in the generated code to have proper type bindings. You probably didn't want
/// to call this directly.
pub fn from_java_static<'j, T: FromJava<'j>>(env: &mut jni::JNIEnv<'j>, val: T::From) -> Result<T, jni::errors::Error> {
	T::from_java(env, val)
}

/// Specifies how a Java type should be converted before being fed to Rust.
pub trait FromJava<'j> : Sized {
	/// The JNI type representing the input.
	type From : Sized;
	/// Attempts to convert this Java object into its Rust counterpart.
	fn from_java(env: &mut jni::JNIEnv<'j>, value: Self::From) -> Result<Self, jni::errors::Error>;
}

macro_rules! auto_from_java {
	($t: ty, $j: ty) => {
		impl<'j> FromJava<'j> for $t {
			type From = $j;
		
			#[inline]
			fn from_java(_: &mut jni::JNIEnv, value: Self::From) -> Result<Self, jni::errors::Error> {
				Ok(value)
			}
		}
	};
}

auto_from_java!(i64, jni::sys::jlong);
auto_from_java!(i32, jni::sys::jint);
auto_from_java!(i16, jni::sys::jshort);
auto_from_java!(i8, jni::sys::jbyte);
auto_from_java!(f32, jni::sys::jfloat);
auto_from_java!(f64, jni::sys::jdouble);
auto_from_java!(JObject<'j>, JObject<'j>);
auto_from_java!(JString<'j>, JString<'j>);
auto_from_java!(JObjectArray<'j>, JObjectArray<'j>);

impl<'j, T: TypeArray> FromJava<'j> for JPrimitiveArray<'j, T> {
	type From = JPrimitiveArray<'j, T>;

	#[inline]
	fn from_java(_: &mut jni::JNIEnv, value: Self::From) -> Result<Self, jni::errors::Error> {
		Ok(value)
	}
}

impl<'j> FromJava<'j> for char {
	type From = jni::sys::jchar;

	#[inline]
	fn from_java(_: &mut jni::JNIEnv, value: Self::From) -> Result<Self, jni::errors::Error> {
		char::from_u32(value.into()).ok_or_else(|| jni::errors::Error::WrongJValueType("char", "invalid u16"))
	}
}

impl<'j> FromJava<'j> for bool {
	type From = jni::sys::jboolean;

	#[inline]
	fn from_java(_: &mut jni::JNIEnv, value: Self::From) -> Result<Self, jni::errors::Error> {
		Ok(value == 1)
	}
}

impl<'j> FromJava<'j> for String {
	type From = JString<'j>;

	fn from_java(env: &mut jni::JNIEnv<'j>, value: Self::From) -> Result<Self, jni::errors::Error> {
		if value.is_null() { return Err(jni::errors::Error::NullPtr("string can't be null")) };
		Ok(env.get_string(&value)?.into())
	}
}

impl<'j, T> FromJava<'j> for Option<T>
where
	T: FromJava<'j, From: AsRef<JObject<'j>>>,
{
	type From = T::From;

	fn from_java(env: &mut jni::JNIEnv<'j>, value: Self::From) -> Result<Self, jni::errors::Error> {
		if value.as_ref().is_null() { return Ok(None) };
		Ok(Some(T::from_java(env, value)?))
	}
}

impl<'j, T: FromJava<'j, From = JObject<'j>>> FromJava<'j> for Vec<T> {
	type From = JObjectArray<'j>;

	fn from_java(env: &mut jni::JNIEnv<'j>, value: Self::From) -> Result<Self, jni::errors::Error> {
		let len = env.get_array_length(&value)?;
		let mut out = Vec::new();
		for i in 0..len {
			let el = env.get_object_array_element(&value, i)?;
			out.push(T::from_java(env, el)?);
		}
		Ok(out)
	}
}

#[cfg(feature = "uuid")]
impl<'j> FromJava<'j> for uuid::Uuid {
	type From = JObject<'j>;
	fn from_java(env: &mut jni::JNIEnv<'j>, uuid: Self::From) -> Result<Self, jni::errors::Error> {
		let lsb = u64::from_ne_bytes(
			env.call_method(&uuid, "getLeastSignificantBits", "()J", &[])?
				.j()?
				.to_ne_bytes()
		);

		let msb = u64::from_ne_bytes(
			env.call_method(&uuid, "getMostSignificantBits", "()J", &[])?
				.j()?
				.to_ne_bytes()
		);
		
		Ok(uuid::Uuid::from_u64_pair(msb, lsb))
	}
}
