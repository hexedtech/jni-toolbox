
/// An error which can be auto-thrown by jni_toolbox
/// all your custom errors must implement Into<JniToolboxError>
/// to be auto-throwable. an exception class must be specified in
/// clazz, and an optional message can be provided
#[derive(thiserror::Error, Debug)]
#[error("{message:?} ({clazz})")]
pub struct JniToolboxError {
	/// an optional message to be thrown with this exception
	pub message: Option<String>,
	/// the exception class to be constructed and thrown
	pub clazz: &'static str,
}

/// The default error policy for **JNI Toolbox**
/// this policy only applies to functions returning [`JniToolboxError`] errors, or anything 
/// that can be converted into it (`impl Into<JniToolboxError>`)
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
		if let Some(msg) = err.message {
			let _ = env.throw_new(
				jni::strings::JNIString::new(err.clazz),
				jni::strings::JNIString::new(msg),
			);
		} else {
			let _ = env.throw_new_void(jni::strings::JNIString::new(err.clazz));
		};
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

impl From<jni::errors::Error> for JniToolboxError {
	fn from(value: jni::errors::Error) -> Self {
		let message = Some(format!("{value} -- {value:?}"));
		let clazz = match value {
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
		};

		Self { message, clazz }
	}
}
