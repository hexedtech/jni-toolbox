
/// A trait to automatically convert errors in java exceptions
/// implementing this trait on your error struct/enum allows
/// jni-toolbox to automatically throw requested exceptions
/// when this error is encountered
pub trait IntoException: std::error::Error {
	/// the exception class to be constructed and thrown
	fn jclass(&self) -> &'static str;

	/// returns an exception message based on Display and Debug (concatenated)
	/// override to set custom exception messages
	fn message(&self) -> String {
		format!("{self} -- {self:?}")
	}
}

/// An error which can be auto-thrown by jni_toolbox
#[derive(Debug, thiserror::Error)]
#[error("{message} ({clazz})")]
pub struct JniToolboxError {
	pub message: String,
	pub clazz: &'static str,
}

impl<T: IntoException> From<T> for JniToolboxError {
	fn from(value: T) -> Self {
		Self {
			clazz: value.jclass(),
			message: value.message(),
		}
	}
}


/// The default error policy for **JNI Toolbox**
/// this policy only applies to functions returning [`JniToolboxError`] errors, or anything 
/// that can be converted into it (implementing [`IntoException`] )
/// in case of error, it will throw a new exception of given class, with given message.
/// class and message are provided by the resulting [`JniToolboxError`]
/// note that all [`jni::errors::Error`] are mapped to [`JniToolboxError`]s
pub struct JniToolboxErrorPolicy;

impl<T: Default> jni::errors::ErrorPolicy<T, JniToolboxError> for JniToolboxErrorPolicy {
	type Captures<'unowned_env_local: 'native_method, 'native_method> = ();

	fn on_error<'unowned_env_local: 'native_method, 'native_method>(
		env: &mut jni::Env<'unowned_env_local>,
		_cap: &mut Self::Captures<'unowned_env_local, 'native_method>,
		err: JniToolboxError,
	) -> jni::errors::Result<T> {
		let _ = env.throw_new(
			jni::strings::JNIString::new(err.clazz),
			jni::strings::JNIString::new(err.message),
		);
		Ok(T::default())
	}

	fn on_panic<'unowned_env_local: 'native_method, 'native_method>(
		env: &mut jni::Env<'unowned_env_local>,
		_cap: &mut Self::Captures<'unowned_env_local, 'native_method>,
		_payload: Box<dyn std::any::Any + Send + 'static>,
	) -> jni::errors::Result<T> {
		let _ = env.throw_new(jni::jni_str!("java/lang/RuntimeException"), jni::jni_str!("panic in native code"));
		Ok(T::default())
	}
}

impl IntoException for jni::errors::Error {
	fn jclass(&self) -> &'static str {
		match self {
			jni::errors::Error::NullPtr(_) => "java/lang/NullPointerException",

			// TODO do we really care about mapping all these?

			// jni::errors::Error::UninitializedJavaVM => todo!(),
			// jni::errors::Error::WrongJValueType(_, _) => todo!(),
			// jni::errors::Error::WrongObjectType => todo!(),
			// jni::errors::Error::InvalidCtorReturn => todo!(),
			// jni::errors::Error::InvalidArgList(runtime_method_signature) => todo!(),
			// jni::errors::Error::ObjectFreed => todo!(),
			// jni::errors::Error::ClassNotFound { name } => todo!(),
			// jni::errors::Error::NoClassDefFound { requested, cause } => todo!(),
			// jni::errors::Error::ClassFormatError => todo!(),
			// jni::errors::Error::ClassCircularityError => todo!(),
			// jni::errors::Error::LinkageError { requested, cause } => todo!(),
			// jni::errors::Error::MethodNotFound { name, sig } => todo!(),
			// jni::errors::Error::NoSuchMethod(_) => todo!(),
			// jni::errors::Error::FieldNotFound { name, sig } => todo!(),
			// jni::errors::Error::JavaException => todo!(),
			// jni::errors::Error::ExceptionInInitializer { exception } => todo!(),
			// jni::errors::Error::Instantiation => todo!(),
			// jni::errors::Error::EnvMethodNotFound(_) => todo!(),
			// jni::errors::Error::TryLock => todo!(),
			// jni::errors::Error::FieldAlreadySet(_) => todo!(),
			// jni::errors::Error::ThrowFailed(_) => todo!(),
			// jni::errors::Error::ParseFailed(_) => todo!(),
			// jni::errors::Error::JniCall(jni_error) => todo!(),
			// jni::errors::Error::InvalidUtf16 { source } => todo!(),
			// jni::errors::Error::InvalidUtf32 { char, source } => todo!(),
			// jni::errors::Error::UnsupportedVersion => todo!(),
			// jni::errors::Error::ThreadAttachmentGuarded => todo!(),
			// jni::errors::Error::CaughtJavaException { exception, name, msg, stack, .. } => todo!(),
			// jni::errors::Error::IndexOutOfBounds => todo!(),
			// jni::errors::Error::IllegalMonitorState => todo!(),
			// jni::errors::Error::SecurityViolation => todo!(),

			_ => "java/lang/RuntimeException",
		}
	}
}
