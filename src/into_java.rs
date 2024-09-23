use jni::objects::{JObject, JObjectArray, JString};

use crate::raw_java::FromJavaRaw;


/// Specifies how a Rust type should be converted into a Java primitive.
pub trait IntoJavaPrimitive<'j> {
	/// The JNI type representing the output.
	type T;
	/// Attempts to convert this Rust object into a Java primitive.
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

/// Specifies how a Rust type should be converted into a Java object.
pub trait IntoJavaObject<'j> {
	type T: std::convert::AsRef<JObject<'j>>;
	/// The Java class associated with this type.
	const CLASS: &'static str;
	/// Attempts to convert this Rust object into a Java object.
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
