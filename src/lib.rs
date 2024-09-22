use jni::{objects::{JObject, JObjectArray, JString}, sys::{jboolean, jobject}};
pub use jni_toolbox_macro::jni;

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
	($t:ty, $j:ty) => {
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
auto_from_java!(f32, jni::sys::jfloat);
auto_from_java!(f64, jni::sys::jdouble);
auto_from_java!(JObject<'j>, JObject<'j>);
auto_from_java!(JObjectArray<'j>, JObjectArray<'j>);

impl<'j> FromJava<'j> for bool {
	type T = jni::sys::jboolean;

	#[inline]
	fn from_java(_: &mut jni::JNIEnv, value: Self::T) -> Result<Self, jni::errors::Error> {
		Ok(value == 1)
	}
}

impl<'j> FromJava<'j> for String {
	type T = jni::objects::JString<'j>;

	fn from_java(env: &mut jni::JNIEnv<'j>, value: Self::T) -> Result<Self, jni::errors::Error> {
		if value.is_null() { return Err(jni::errors::Error::NullPtr("string can't be null")) };
		Ok(env.get_string(&value)?.into())
	}
}

impl<'j, T: FromJava<'j, T = jni::objects::JObject<'j>>> FromJava<'j> for Option<T> {
	type T = jni::objects::JObject<'j>;

	fn from_java(env: &mut jni::JNIEnv<'j>, value: Self::T) -> Result<Self, jni::errors::Error> {
		if value.is_null() { return Ok(None) };
		Ok(Some(T::from_java(env, value)?))
	}
}

#[cfg(feature = "uuid")]
impl<'j> FromJava<'j> for uuid::Uuid {
	type T = jni::objects::JObject<'j>;
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

pub trait IntoJava<'j> {
	type T;

	fn into_java(self, env: &mut jni::JNIEnv<'j>) -> Result<Self::T, jni::errors::Error>;
}

macro_rules! auto_into_java {
	($t:ty, $j:ty) => {
		impl<'j> IntoJava<'j> for $t {
			type T = $j;
		
			fn into_java(self, _: &mut jni::JNIEnv<'j>) -> Result<Self::T, jni::errors::Error> {
				Ok(self)
			}
		}
	};
}

auto_into_java!(i64, jni::sys::jlong);
auto_into_java!(i32, jni::sys::jint);
auto_into_java!(i16, jni::sys::jshort);
auto_into_java!(f32, jni::sys::jfloat);
auto_into_java!(f64, jni::sys::jdouble);
auto_into_java!((), ());

impl<'j> IntoJava<'j> for bool {
	type T = jni::sys::jboolean;

	#[inline]
	fn into_java(self, _: &mut jni::JNIEnv) -> Result<Self::T, jni::errors::Error> {
		Ok(if self { 1 } else { 0 })
	}
}

impl<'j> IntoJava<'j> for &str {
	type T = jni::sys::jstring;
	fn into_java(self, env: &mut jni::JNIEnv<'j>) -> Result<Self::T, jni::errors::Error> {
		Ok(env.new_string(self)?.as_raw())
	}
}

impl<'j> IntoJava<'j> for String {
	type T = jni::sys::jstring;
	fn into_java(self, env: &mut jni::JNIEnv<'j>) -> Result<Self::T, jni::errors::Error> {
		self.as_str().into_java(env)
	}
}

impl<'j> IntoJava<'j> for Vec<String> {
	type T = jni::sys::jobjectArray;

	fn into_java(self, env: &mut jni::JNIEnv<'j>) -> Result<Self::T, jni::errors::Error> {
		let mut array = env.new_object_array(self.len() as i32, "java/lang/String", JObject::null())?;
		for (n, el) in self.into_iter().enumerate() {
			let string = env.new_string(el)?;
			env.set_object_array_element(&mut array, n as i32, string)?;
		}
		Ok(array.into_raw())
	}
}

impl<'j, T: IntoJava<'j, T = jni::sys::jobject>> IntoJava<'j> for Option<T> {
	type T = T::T;
	fn into_java(self, env: &mut jni::JNIEnv<'j>) -> Result<Self::T, jni::errors::Error> {
		match self {
			Some(x) => x.into_java(env),
			None => Ok(std::ptr::null_mut()),
		}
	}
}

#[cfg(feature = "uuid")]
impl<'j> IntoJava<'j> for uuid::Uuid {
	type T = jni::sys::jobject;
	fn into_java(self, env: &mut jni::JNIEnv<'j>) -> Result<Self::T, jni::errors::Error> {
		let class = env.find_class("java/util/UUID")?;
		let (msb, lsb) = self.as_u64_pair();
		let msb = i64::from_ne_bytes(msb.to_ne_bytes());
		let lsb = i64::from_ne_bytes(lsb.to_ne_bytes());
		env.new_object(&class, "(JJ)V", &[jni::objects::JValueGen::Long(msb), jni::objects::JValueGen::Long(lsb)])
			.map(|j| j.as_raw())
	}
}
