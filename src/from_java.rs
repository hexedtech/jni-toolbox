use jni::objects::{JObject, JString};


/// Used in the generated code to have proper type bindings. You probably didn't want
/// to call this directly.
pub fn from_java_static<'j, T: FromJava<'j>>(env: &mut jni::JNIEnv<'j>, val: T::T) -> Result<T, jni::errors::Error> {
	T::from_java(env, val)
}

/// Specifies how a Java type should be converted before being fed to Rust.
pub trait FromJava<'j> : Sized {
	/// The JNI type representing the input.
	type T : Sized;
	/// Attempts to convert this Java object into its Rust counterpart.
	fn from_java(env: &mut jni::JNIEnv<'j>, value: Self::T) -> Result<Self, jni::errors::Error>;
}

macro_rules! auto_from_java {
	($t: ty, $j: ty) => {
		impl<'j> FromJava<'j> for $t {
			type T = $j;
		
			#[inline]
			fn from_java(_: &mut jni::JNIEnv, value: Self::T) -> Result<Self, jni::errors::Error> {
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

impl<'j> FromJava<'j> for bool {
	type T = jni::sys::jboolean;

	#[inline]
	fn from_java(_: &mut jni::JNIEnv, value: Self::T) -> Result<Self, jni::errors::Error> {
		Ok(value == 1)
	}
}

impl<'j> FromJava<'j> for String {
	type T = JString<'j>;

	fn from_java(env: &mut jni::JNIEnv<'j>, value: Self::T) -> Result<Self, jni::errors::Error> {
		if value.is_null() { return Err(jni::errors::Error::NullPtr("string can't be null")) };
		Ok(env.get_string(&value)?.into())
	}
}

impl<'j, T: FromJava<'j, T: std::convert::AsRef<JObject<'j>>>> FromJava<'j> for Option<T> {
	type T = T::T;

	fn from_java(env: &mut jni::JNIEnv<'j>, value: Self::T) -> Result<Self, jni::errors::Error> {
		if value.as_ref().is_null() { return Ok(None) };
		Ok(Some(T::from_java(env, value)?))
	}
}

#[cfg(feature = "uuid")]
impl<'j> FromJava<'j> for uuid::Uuid {
	type T = JObject<'j>;
	fn from_java(env: &mut jni::JNIEnv<'j>, uuid: Self::T) -> Result<Self, jni::errors::Error> {
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
