use jni::objects::JObject;


/// Specifies how a Rust type should be converted into a Java primitive.
pub trait IntoJava<'j> {
	/// The JNI type representing the output.
	type Ret;
	/// Attempts to convert this Rust object into a Java primitive.
	fn into_java(self, _: &mut jni::JNIEnv<'j>) -> Result<Self::Ret, jni::errors::Error>;
}

macro_rules! auto_into_java {
	($t: ty, $j: ty) => {
		impl<'j> IntoJava<'j> for $t {
			type Ret = $j;
		
			#[inline]
			fn into_java(self, _: &mut jni::JNIEnv<'j>) -> Result<Self::Ret, jni::errors::Error> {
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

impl<'j> IntoJava<'j> for bool {
	type Ret = jni::sys::jboolean;

	#[inline]
	fn into_java(self, _: &mut jni::JNIEnv) -> Result<Self::Ret, jni::errors::Error> {
		Ok(if self { 1 } else { 0 })
	}
}

impl<'j, X: IntoJavaObject<'j>> IntoJava<'j> for X {
	type Ret = jni::sys::jobject;

	#[inline]
	fn into_java(self, env: &mut jni::JNIEnv<'j>) -> Result<Self::Ret, jni::errors::Error> {
		Ok(self.into_java_object(env)?.as_raw())
	}
}

/// Specifies how a Rust type should be converted into a Java object.
pub trait IntoJavaObject<'j> {
	/// The Java class associated with this type.
	const CLASS: &'static str;
	/// Attempts to convert this Rust object into a Java object.
	fn into_java_object(self, env: &mut jni::JNIEnv<'j>) -> Result<JObject<'j>, jni::errors::Error>;
}

impl<'j> IntoJavaObject<'j> for JObject<'j> {
	const CLASS: &'static str = "java/lang/Object";
	#[inline]
	fn into_java_object(self, _: &mut jni::JNIEnv<'j>) -> Result<JObject<'j>, jni::errors::Error> {
		Ok(self)
	}
}

macro_rules! auto_into_java_object {
	($t:ty, $cls:literal) => {
		impl<'j> IntoJavaObject<'j> for $t {
			const CLASS: &'static str = $cls;
			#[inline]
			fn into_java_object(self, _: &mut jni::JNIEnv<'j>) -> Result<JObject<'j>, jni::errors::Error> {
				Ok(self.into())
			}
		}
	};
}

auto_into_java_object!(jni::objects::JString<'j>, "java/lang/String");
auto_into_java_object!(jni::objects::JObjectArray<'j>, "java/lang/Object[]");
auto_into_java_object!(jni::objects::JIntArray<'j>, "java/lang/Integer[]");
auto_into_java_object!(jni::objects::JLongArray<'j>, "java/lang/Long[]");
auto_into_java_object!(jni::objects::JShortArray<'j>, "java/lang/Short[]");
auto_into_java_object!(jni::objects::JByteArray<'j>, "java/lang/Byte[]");
auto_into_java_object!(jni::objects::JCharArray<'j>, "java/lang/Char[]");
auto_into_java_object!(jni::objects::JFloatArray<'j>, "java/lang/Float[]");
auto_into_java_object!(jni::objects::JDoubleArray<'j>, "java/lang/Double[]");
auto_into_java_object!(jni::objects::JBooleanArray<'j>, "java/lang/Boolean[]");


impl<'j> IntoJavaObject<'j> for &str {
	const CLASS: &'static str = "java/lang/String";
	fn into_java_object(self, env: &mut jni::JNIEnv<'j>) -> Result<JObject<'j>, jni::errors::Error> {
		Ok(env.new_string(self)?.into())
	}
}

impl<'j> IntoJavaObject<'j> for String {
	const CLASS: &'static str = "java/lang/String";
	fn into_java_object(self, env: &mut jni::JNIEnv<'j>) -> Result<JObject<'j>, jni::errors::Error> {
		self.as_str().into_java_object(env)
	}
}

impl<'j, T: IntoJavaObject<'j>> IntoJavaObject<'j> for Vec<T> {
	const CLASS: &'static str = T::CLASS; // TODO shouldnt it be 'Object[]' ?
	fn into_java_object(self, env: &mut jni::JNIEnv<'j>) -> Result<JObject<'j>, jni::errors::Error> {
		let mut array = env.new_object_array(self.len() as i32, T::CLASS, JObject::null())?;
		for (n, el) in self.into_iter().enumerate() {
			let el = el.into_java_object(env)?;
			env.set_object_array_element(&mut array, n as i32, &el)?;
		}
		Ok(array.into())
	}
}

impl<'j, T: IntoJavaObject<'j>> IntoJavaObject<'j> for Option<T> {
	const CLASS: &'static str = T::CLASS;
	fn into_java_object(self, env: &mut jni::JNIEnv<'j>) -> Result<JObject<'j>, jni::errors::Error> {
		match self {
			Some(x) => x.into_java_object(env),
			None => Ok(JObject::null())
		}
	}
}

macro_rules! auto_into_java_object_primitive_array {
	($t:ty, $fn_new:ident, $fn_set:ident, $clazz:literal) => {
		impl<'j> IntoJavaObject<'j> for Vec<$t> {
			const CLASS: &'static str = $clazz;
			fn into_java_object(self, env: &mut jni::JNIEnv<'j>) -> Result<JObject<'j>, jni::errors::Error> {
				let len = self.len()
					.try_into()
					.map_err(|_| jni::errors::Error::JniCall(jni::errors::JniError::InvalidArguments))?;
				let mut array = env.$fn_new(len)?;
				env.$fn_set(&mut array, 0, self.as_slice())?;
				Ok(array.into())
			}
		}
	};
}

auto_into_java_object_primitive_array!(i8, new_byte_array, set_byte_array_region, "java/lang/Byte[]");
auto_into_java_object_primitive_array!(i16, new_short_array, set_short_array_region, "java/lang/Short[]");
auto_into_java_object_primitive_array!(i32, new_int_array, set_int_array_region, "java/lang/Integer[]");
auto_into_java_object_primitive_array!(i64, new_long_array, set_long_array_region, "java/lang/Long[]");
auto_into_java_object_primitive_array!(f32, new_float_array, set_float_array_region, "java/lang/Float[]");
auto_into_java_object_primitive_array!(f64, new_double_array, set_double_array_region, "java/lang/Double[]");

impl<'j> IntoJavaObject<'j> for Vec<bool> {
	const CLASS: &'static str = "java/lang/Boolean[]";

	fn into_java_object(self, env: &mut jni::JNIEnv<'j>) -> Result<JObject<'j>, jni::errors::Error> {
		let len = self.len()
			.try_into()
			.map_err(|_| jni::errors::Error::JniCall(jni::errors::JniError::InvalidArguments))?;
		let mut array = env.new_boolean_array(len)?;
		let new_self : Vec<u8> = self.into_iter().map(|x| if x { 1 } else { 0 }).collect();
		env.set_boolean_array_region(&mut array, 0, new_self.as_slice())?;
		Ok(array.into())
	}
}

impl<'j> IntoJavaObject<'j> for Vec<char> {
	const CLASS: &'static str = "java/lang/Char[]";

	fn into_java_object(self, env: &mut jni::JNIEnv<'j>) -> Result<JObject<'j>, jni::errors::Error> {
		let len = self.len()
			.try_into()
			.map_err(|_| jni::errors::Error::JniCall(jni::errors::JniError::InvalidArguments))?;
		let mut array = env.new_char_array(len)?;
		let mut new_self : Vec<u16> = Vec::new();
		for c in self {
			new_self.push(
				c
					.try_into()
					.map_err(|_| jni::errors::Error::JniCall(jni::errors::JniError::InvalidArguments))?
			);
		}
		env.set_char_array_region(&mut array, 0, new_self.as_slice())?;
		Ok(array.into())
	}
}

#[cfg(feature = "uuid")]
impl<'j> IntoJavaObject<'j> for uuid::Uuid {
	const CLASS: &'static str = "java/util/UUID";
	fn into_java_object(self, env: &mut jni::JNIEnv<'j>) -> Result<JObject<'j>, jni::errors::Error> {
		let class = env.find_class(Self::CLASS)?;
		let (msb, lsb) = self.as_u64_pair();
		let msb = i64::from_ne_bytes(msb.to_ne_bytes());
		let lsb = i64::from_ne_bytes(lsb.to_ne_bytes());
		env.new_object(&class, "(JJ)V", &[jni::objects::JValueGen::Long(msb), jni::objects::JValueGen::Long(lsb)])
	}
}
