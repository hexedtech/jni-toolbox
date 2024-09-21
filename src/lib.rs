use jni::objects::JString;
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




impl<'j> FromJava<'j> for bool {
	type T = jni::sys::jboolean;

	#[inline]
	fn from_java(_: &mut jni::JNIEnv, value: Self::T) -> Result<Self, jni::errors::Error> {
		Ok(value == 1)
	}
}

impl<'j> FromJava<'j> for i64 {
	type T = jni::sys::jlong;

	#[inline]
	fn from_java(_: &mut jni::JNIEnv, value: Self::T) -> Result<Self, jni::errors::Error> {
		Ok(value)
	}
}

impl<'j> FromJava<'j> for i32 {
	type T = jni::sys::jint;

	#[inline]
	fn from_java(_: &mut jni::JNIEnv, value: Self::T) -> Result<Self, jni::errors::Error> {
		Ok(value)
	}
}

impl<'j> FromJava<'j> for i16 {
	type T = jni::sys::jshort;

	#[inline]
	fn from_java(_: &mut jni::JNIEnv, value: Self::T) -> Result<Self, jni::errors::Error> {
		Ok(value)
	}
}

impl<'j> FromJava<'j> for f32 {
	type T = jni::sys::jfloat;

	#[inline]
	fn from_java(_: &mut jni::JNIEnv, value: Self::T) -> Result<Self, jni::errors::Error> {
	  Ok(value)
	}
}

impl<'j> FromJava<'j> for f64 {
	type T = jni::sys::jdouble;

	#[inline]
	fn from_java(_: &mut jni::JNIEnv, value: Self::T) -> Result<Self, jni::errors::Error> {
		Ok(value)
	}
}

impl<'j> FromJava<'j> for String {
	type T = jni::objects::JString<'j>;

	fn from_java(env: &mut jni::JNIEnv<'j>, value: Self::T) -> Result<Self, jni::errors::Error> {
		if value.is_null() { return Err(jni::errors::Error::NullPtr("string can't be null")) };
		Ok(env.get_string(&value)?.into())
	}
}

impl<'j> FromJava<'j> for Option<String> {
	type T = jni::objects::JString<'j>;

	fn from_java(env: &mut jni::JNIEnv<'j>, value: Self::T) -> Result<Self, jni::errors::Error> {
		if value.is_null() { return Ok(None) };
		Ok(Some(String::from_java(env, value)?))
	}
}












pub trait IntoJava<'j> {
	type T;

	fn into_java(self, env: &mut jni::JNIEnv<'j>) -> Result<Self::T, jni::errors::Error>;
}

impl<'j> IntoJava<'j> for String {
	type T = JString<'j>;

	fn into_java(self, env: &mut jni::JNIEnv<'j>) -> Result<Self::T, jni::errors::Error> {
		env.new_string(self)
	}
}
