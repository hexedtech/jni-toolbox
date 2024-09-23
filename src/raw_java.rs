use jni::objects::{JObject, JObjectArray, JString};


/// Needed internally to perform some operations.
pub trait FromJavaRaw {
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
