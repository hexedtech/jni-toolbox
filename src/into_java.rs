use jni::{jni_str, objects::{JObject, JObjectArray}, signature::{JavaType, Primitive, RuntimeMethodSignature}};


/// Specifies how a Rust type should be converted into a Java primitive.
pub trait IntoJava<'j> {
	/// The JNI-compatible type the conversion will return.
	type Ret;
	/// The JNI type representing the output.
	fn signature() -> (String, JavaType);
	/// Attempts to convert this Rust object into a Java primitive.
	fn into_java(self, e: &mut jni::Env<'j>) -> Result<Self::Ret, jni::errors::Error>;
	/// Attempts to convert this Rust object into a JValue (used in constructors)
	fn into_jvalue(self, e: &mut jni::Env<'j>) -> Result<jni::JValueOwned<'j>, jni::errors::Error>;
}

impl<'j> IntoJava<'j> for () {
	type Ret = ();

	fn signature() -> (String, JavaType) {
		("V".into(), JavaType::Primitive(Primitive::Void))
	}

	#[inline]
	fn into_java(self, _: &mut jni::Env<'j>) -> Result<Self::Ret, jni::errors::Error> {
		Ok(self)
	}

	fn into_jvalue(self, _: &mut jni::Env<'j>) -> Result<jni::JValueOwned<'j>, jni::errors::Error> {
		Ok(jni::JValueOwned::Void)
	}
}

macro_rules! auto_into_java {
	($t: ty, $sig:literal, $j:ty, $jt:expr, $jvalue:expr) => {
		impl<'j> IntoJava<'j> for $t {
			type Ret = $j;

			fn signature() -> (String, JavaType) {
				($sig.into(), $jt)
			}
		
			#[inline]
			fn into_java(self, _: &mut jni::Env<'j>) -> Result<Self::Ret, jni::errors::Error> {
				Ok(self)
			}

			fn into_jvalue(self, _: &mut jni::Env<'j>) -> Result<jni::JValueOwned<'j>, jni::errors::Error> {
				Ok($jvalue(self))
			}
		}
	};
}

auto_into_java!(i64, "L", jni::sys::jlong, JavaType::Primitive(Primitive::Long), jni::JValueOwned::Long);
auto_into_java!(i32, "I", jni::sys::jint, JavaType::Primitive(Primitive::Int), jni::JValueOwned::Int);
auto_into_java!(i16, "S", jni::sys::jshort, JavaType::Primitive(Primitive::Short), jni::JValueOwned::Short);
auto_into_java!(i8, "B", jni::sys::jbyte, JavaType::Primitive(Primitive::Byte), jni::JValueOwned::Byte);
auto_into_java!(f32, "F", jni::sys::jfloat, JavaType::Primitive(Primitive::Float), jni::JValueOwned::Float);
auto_into_java!(f64, "D", jni::sys::jdouble, JavaType::Primitive(Primitive::Double), jni::JValueOwned::Double);
auto_into_java!(bool, "Z", jni::sys::jboolean, JavaType::Primitive(Primitive::Boolean), jni::JValueOwned::Bool);

impl<'j, X: IntoJavaObject<'j>> IntoJava<'j> for X {
	type Ret = jni::sys::jobject;

	fn signature() -> (String, JavaType) {
		let jt = if Self::ARRAY_DEPTH > 0 {
			JavaType::Array
		} else {
			JavaType::Object
		};
	
		(format!("{}L{};", "[".repeat(Self::ARRAY_DEPTH), Self::CLASS), jt)
	}

	#[inline]
	fn into_java(self, env: &mut jni::Env<'j>) -> Result<Self::Ret, jni::errors::Error> {
		Ok(self.into_java_object(env)?.as_raw())
	}

	fn into_jvalue(self, e: &mut jni::Env<'j>) -> Result<jni::JValueOwned<'j>, jni::errors::Error> {
		Ok(jni::JValueOwned::Object(self.into_java_object(e)?))
	}
}

/// Specifies how a Rust type should be converted into a Java object.
pub trait IntoJavaObject<'j> {
	/// The Java class associated with this type.
	const CLASS: &'static str;
	/// The array depth of this type. It is always 0 for non-array types.
	const ARRAY_DEPTH: usize = 0;
	/// Attempts to convert this Rust object into a Java object.
	fn into_java_object(self, env: &mut jni::Env<'j>) -> Result<JObject<'j>, jni::errors::Error>;
}

impl<'j> IntoJavaObject<'j> for JObject<'j> {
	const CLASS: &'static str = "java/lang/Object";

	#[inline]
	fn into_java_object(self, _: &mut jni::Env<'j>) -> Result<JObject<'j>, jni::errors::Error> {
		Ok(self)
	}
}

macro_rules! auto_into_java_object {
	($t:ty, $depth:literal, $cls:literal) => {
		impl<'j> IntoJavaObject<'j> for $t {
			const ARRAY_DEPTH: usize = $depth;
			const CLASS: &'static str = $cls;
			#[inline]
			fn into_java_object(self, _: &mut jni::Env<'j>) -> Result<JObject<'j>, jni::errors::Error> {
				Ok(self.into())
			}
		}
	};
}

auto_into_java_object!(jni::objects::JString<'j>, 0, "java/lang/String");
auto_into_java_object!(jni::objects::JObjectArray<'j>, 1, "java/lang/Object");
auto_into_java_object!(jni::objects::JIntArray<'j>, 1, "java/lang/Integer");
auto_into_java_object!(jni::objects::JLongArray<'j>, 1, "java/lang/Long");
auto_into_java_object!(jni::objects::JShortArray<'j>, 1, "java/lang/Short");
auto_into_java_object!(jni::objects::JByteArray<'j>, 1, "java/lang/Byte");
auto_into_java_object!(jni::objects::JCharArray<'j>, 1, "java/lang/Char");
auto_into_java_object!(jni::objects::JFloatArray<'j>, 1, "java/lang/Float");
auto_into_java_object!(jni::objects::JDoubleArray<'j>, 1, "java/lang/Double");
auto_into_java_object!(jni::objects::JBooleanArray<'j>, 1, "java/lang/Boolean");

impl<'j> IntoJavaObject<'j> for &str {
	const CLASS: &'static str = "java/lang/String";
	fn into_java_object(self, env: &mut jni::Env<'j>) -> Result<JObject<'j>, jni::errors::Error> {
		Ok(env.new_string(self)?.into())
	}
}

impl<'j> IntoJavaObject<'j> for String {
	const CLASS: &'static str = "java/lang/String";
	fn into_java_object(self, env: &mut jni::Env<'j>) -> Result<JObject<'j>, jni::errors::Error> {
		self.as_str().into_java_object(env)
	}
}

impl<'j, T: IntoJavaObject<'j>> IntoJavaObject<'j> for Option<T> {
	const CLASS: &'static str = T::CLASS;
	fn into_java_object(self, env: &mut jni::Env<'j>) -> Result<JObject<'j>, jni::errors::Error> {
		match self {
			Some(x) => x.into_java_object(env),
			None => Ok(JObject::null())
		}
	}
}

macro_rules! auto_into_java_object_primitive_option {
	($t:ty, $clazz:literal, $primitive_desc:literal) => {
		impl<'j> IntoJavaObject<'j> for Option<$t> {
			const CLASS: &'static str = $clazz;
			fn into_java_object(self, env: &mut jni::Env<'j>) -> Result<JObject<'j>, jni::errors::Error> {
				match self {
					Some(val) => {
						let class_name = jni::strings::JNIString::new(Self::CLASS);
						let class = env.find_class(&class_name)?;
						let jvalue = val.into_jvalue(env)?;
						let sig = RuntimeMethodSignature::from_str(concat!("(", $primitive_desc, ")L", $clazz, ";"))?;
						let res = env.call_static_method(
							&class,
							jni_str!("valueOf"),
							sig.method_signature(),
							&[jvalue.borrow()]
						)?;
						res.l()
					},
					None => Ok(JObject::null()),
				}
			}
		}
	};
}

auto_into_java_object_primitive_option!(i8, "java/lang/Byte", "B");
auto_into_java_object_primitive_option!(i16, "java/lang/Short", "S");
auto_into_java_object_primitive_option!(i32, "java/lang/Integer", "I");
auto_into_java_object_primitive_option!(i64, "java/lang/Long", "J");
auto_into_java_object_primitive_option!(f32, "java/lang/Float", "F");
auto_into_java_object_primitive_option!(f64, "java/lang/Double", "D");
auto_into_java_object_primitive_option!(bool, "java/lang/Boolean", "Z");

impl<'j, T: IntoJavaObject<'j>> IntoJavaObject<'j> for Vec<T> {
	const ARRAY_DEPTH: usize = 1;
	const CLASS: &'static str = T::CLASS;
	fn into_java_object(self, env: &mut jni::Env<'j>) -> Result<JObject<'j>, jni::errors::Error> {
		let arr: JObjectArray<'j, JObject<'j>> = JObjectArray::<JObject<'j>>::new(env, self.len(), JObject::null())?;
		for (n, el) in self.into_iter().enumerate() {
			let el = el.into_java_object(env)?;
			arr.set_element(env, n, &el)?;
		}
		Ok(JObject::from(arr))
	}
}

macro_rules! auto_into_java_object_primitive_array {
	($t:ty, $fn_new:ident, $clazz:literal) => {
		impl<'j> IntoJavaObject<'j> for Vec<$t> {
			const ARRAY_DEPTH: usize = 1;
			const CLASS: &'static str = $clazz;
			fn into_java_object(self, env: &mut jni::Env<'j>) -> Result<JObject<'j>, jni::errors::Error> {
				let array = env.$fn_new(self.len())?;
				array.set_region(env, 0, self.as_slice())?;
				Ok(array.into())
			}
		}
	};
}

auto_into_java_object_primitive_array!(i8, new_byte_array, "java/lang/Byte");
auto_into_java_object_primitive_array!(i16, new_short_array, "java/lang/Short");
auto_into_java_object_primitive_array!(i32, new_int_array, "java/lang/Integer");
auto_into_java_object_primitive_array!(i64, new_long_array, "java/lang/Long");
auto_into_java_object_primitive_array!(f32, new_float_array, "java/lang/Float");
auto_into_java_object_primitive_array!(f64, new_double_array, "java/lang/Double");
auto_into_java_object_primitive_array!(bool, new_boolean_array, "java/lang/Boolean");

impl<'j> IntoJavaObject<'j> for Vec<u8> {
	const ARRAY_DEPTH: usize = 1;
	const CLASS: &'static str = "java/lang/Byte";
	fn into_java_object(self, env: &mut jni::Env<'j>) -> Result<JObject<'j>, jni::errors::Error> {
		let array = env.new_byte_array(self.len())?;
		let transmuted = self.into_iter().map(|x| x as i8).collect::<Vec<i8>>();
		array.set_region(env, 0, transmuted.as_slice())?;
		Ok(array.into())
	}
}

impl<'j> IntoJavaObject<'j> for Vec<char> {
	const ARRAY_DEPTH: usize = 1;
	const CLASS: &'static str = "java/lang/Character";
	fn into_java_object(self, env: &mut jni::Env<'j>) -> Result<JObject<'j>, jni::errors::Error> {
		let array = env.new_char_array(self.len())?;
		let mut new_self : Vec<u16> = Vec::new();
		for c in self {
			new_self.push(
				c
					.try_into()
					.map_err(|_| jni::errors::Error::JniCall(jni::errors::JniError::InvalidArguments))?
			);
		}
		array.set_region(env, 0, new_self.as_slice())?;
		Ok(array.into())
	}
}

#[cfg(feature = "uuid")]
impl<'j> IntoJavaObject<'j> for uuid::Uuid {
	const CLASS: &'static str = "java/util/UUID";
	fn into_java_object(self, env: &mut jni::Env<'j>) -> Result<JObject<'j>, jni::errors::Error> {
		let class_name = jni::strings::JNIString::new(Self::CLASS);
		let class = env.find_class(&class_name)?;
		let (msb, lsb) = self.as_u64_pair();
		let msb = i64::from_ne_bytes(msb.to_ne_bytes());
		let lsb = i64::from_ne_bytes(lsb.to_ne_bytes());
		env.new_object(&class, jni::jni_sig!("(JJ)V"), &[jni::JValue::Long(msb), jni::JValue::Long(lsb)])
	}
}
