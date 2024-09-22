pub use jni_toolbox_macro::jni;
use jni::objects::{JObject, JObjectArray, JString};

pub trait JniToolboxError: std::error::Error {
	fn jclass(&self) -> String;
}

impl JniToolboxError for jni::errors::Error {
	fn jclass(&self) -> String {
		"java/lang/RuntimeException".to_string()
	}
}

impl JniToolboxError for jni::errors::JniError {
	fn jclass(&self) -> String {
		"java/lang/RuntimeException".to_string()
	}
}

pub fn from_java_static<'j, T: FromJava<'j>>(env: &mut jni::JNIEnv<'j>, val: T::T) -> Result<T, jni::errors::Error> {
	T::from_java(env, val)
}

pub trait FromJava<'j> : Sized {
	type T : Sized;
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

pub trait IntoJavaPrimitive<'j> {
	type T;
	fn into_java(self, _: &mut jni::JNIEnv<'j>) -> Result<Self::T, jni::errors::Error>;
}

macro_rules! auto_into_java {
	($t: ty, $j: ty) => {
		impl<'j> IntoJavaPrimitive<'j> for $t {
			type T = $j;
		
			fn into_java(self, _: &mut jni::JNIEnv<'j>) -> Result<Self::T, jni::errors::Error> {
				Ok(self)
			}
		}
	};
}

// TODO: primitive arrays!

auto_into_java!(i64, jni::sys::jlong);
auto_into_java!(i32, jni::sys::jint);
auto_into_java!(i16, jni::sys::jshort);
auto_into_java!(i8, jni::sys::jbyte);
auto_into_java!(f32, jni::sys::jfloat);
auto_into_java!(f64, jni::sys::jdouble);
auto_into_java!((), ());

impl<'j> IntoJavaPrimitive<'j> for bool {
	type T = jni::sys::jboolean;

	#[inline]
	fn into_java(self, _: &mut jni::JNIEnv) -> Result<Self::T, jni::errors::Error> {
		Ok(if self { 1 } else { 0 })
	}
}

pub trait IntoJavaObject<'j> {
	type T: std::convert::AsRef<JObject<'j>>;
	const CLASS: &'static str;
	fn into_java(self, env: &mut jni::JNIEnv<'j>) -> Result<Self::T, jni::errors::Error>;
}

impl<'j> IntoJavaObject<'j> for &str {
	type T = JString<'j>;
	const CLASS: &'static str = "java/lang/String";
	fn into_java(self, env: &mut jni::JNIEnv<'j>) -> Result<Self::T, jni::errors::Error> {
		env.new_string(self)
	}
}

impl<'j> IntoJavaObject<'j> for String {
	type T = JString<'j>;
	const CLASS: &'static str = "java/lang/String";
	fn into_java(self, env: &mut jni::JNIEnv<'j>) -> Result<Self::T, jni::errors::Error> {
		self.as_str().into_java(env)
	}
}

impl<'j, E: IntoJavaObject<'j>> IntoJavaObject<'j> for Vec<E> {
	type T = JObjectArray<'j>;
	const CLASS: &'static str = E::CLASS;
	fn into_java(self, env: &mut jni::JNIEnv<'j>) -> Result<Self::T, jni::errors::Error> {
		let mut array = env.new_object_array(self.len() as i32, E::CLASS, JObject::null())?;
		for (n, el) in self.into_iter().enumerate() {
			let el = el.into_java(env)?;
			env.set_object_array_element(&mut array, n as i32, &el)?;
		}
		Ok(array)
	}
}

impl<'j, E: std::convert::AsRef<JObject<'j>> + FromJavaRaw, T: IntoJavaObject<'j, T = E>> IntoJavaObject<'j> for Option<T> {
	type T = E;
	const CLASS: &'static str = T::CLASS;
	fn into_java(self, env: &mut jni::JNIEnv<'j>) -> Result<Self::T, jni::errors::Error> {
		match self {
			Some(x) => x.into_java(env),
			None => Ok(unsafe { E::from_java_raw(std::ptr::null_mut()) }) // safe, that's what JObject::null does
		}
	}
}

#[cfg(feature = "uuid")]
impl<'j> IntoJavaObject<'j> for uuid::Uuid {
	type T = jni::objects::JObject<'j>;
	const CLASS: &'static str = "java/util/UUID";
	fn into_java(self, env: &mut jni::JNIEnv<'j>) -> Result<Self::T, jni::errors::Error> {
		let class = env.find_class(Self::CLASS)?;
		let (msb, lsb) = self.as_u64_pair();
		let msb = i64::from_ne_bytes(msb.to_ne_bytes());
		let lsb = i64::from_ne_bytes(lsb.to_ne_bytes());
		env.new_object(&class, "(JJ)V", &[jni::objects::JValueGen::Long(msb), jni::objects::JValueGen::Long(lsb)])
	}
}

/// Needed internally to perform some operations.
trait FromJavaRaw {
	unsafe fn from_java_raw(raw: jni::sys::jobject) -> Self;
}

macro_rules! auto_from_raw {
	($type: ty) => {
		impl FromJavaRaw for $type {
			#[inline]
			unsafe fn from_java_raw(raw: jni::sys::jobject) -> Self {
				Self::from_raw(raw)
			}
		}
	};
}

auto_from_raw!(JObject<'_>);
auto_from_raw!(JString<'_>);
auto_from_raw!(JObjectArray<'_>);

/// Intermediate trait used to guess the JNI return type.
/// Usually doesn't need to be manually implemented.
pub trait IntoJavaRaw {
	type T;
	fn into_java_raw(self) -> Self::T;
}

macro_rules! auto_into_raw_primitive {
	($type: ty) => {
		impl IntoJavaRaw for $type {
			type T = $type;
			fn into_java_raw(self) -> Self::T {
				self
			}
		}
	}
}

auto_into_raw_primitive!(jni::sys::jlong);
auto_into_raw_primitive!(jni::sys::jint);
auto_into_raw_primitive!(jni::sys::jshort);
auto_into_raw_primitive!(jni::sys::jbyte);
auto_into_raw_primitive!(jni::sys::jdouble);
auto_into_raw_primitive!(jni::sys::jfloat);
auto_into_raw_primitive!(jni::sys::jboolean);
auto_into_raw_primitive!(());

macro_rules! auto_into_raw_object {
	($lt: lifetime, $type: ty) => {
		impl<'j> IntoJavaRaw for $type {
			type T = jni::sys::jobject;
			fn into_java_raw(self) -> Self::T {
				self.as_raw()
			}
		}
	};
}

auto_into_raw_object!('j, JObject<'j>);
auto_into_raw_object!('j, JString<'j>);
auto_into_raw_object!('j, JObjectArray<'j>);
