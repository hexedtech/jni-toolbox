pub mod into_java;
pub mod from_java;

pub use jni_toolbox_macro::jni;
pub use into_java::{IntoJavaObject, IntoJava};
pub use from_java::{FromJava, from_java_static};


/// An error that is meant to be used with jni-toolbox.
pub trait JniToolboxError: std::error::Error {
	/// The Java class for the matching exception.
	fn jclass(&self) -> String;
}

impl JniToolboxError for jni::errors::Error {
	fn jclass(&self) -> String {
		match self {
			jni::errors::Error::NullPtr(_) => "java/lang/NullPointerException",
			_ => "java/lang/RuntimeException",
			// jni::errors::Error::WrongJValueType(_, _) => todo!(),
			// jni::errors::Error::InvalidCtorReturn => todo!(),
			// jni::errors::Error::InvalidArgList(_) => todo!(),
			// jni::errors::Error::MethodNotFound { name, sig } => todo!(),
			// jni::errors::Error::FieldNotFound { name, sig } => todo!(),
			// jni::errors::Error::JavaException => todo!(),
			// jni::errors::Error::JNIEnvMethodNotFound(_) => todo!(),
			// jni::errors::Error::NullDeref(_) => todo!(),
			// jni::errors::Error::TryLock => todo!(),
			// jni::errors::Error::JavaVMMethodNotFound(_) => todo!(),
			// jni::errors::Error::FieldAlreadySet(_) => todo!(),
			// jni::errors::Error::ThrowFailed(_) => todo!(),
			// jni::errors::Error::ParseFailed(_, _) => todo!(),
			// jni::errors::Error::JniCall(_) => todo!(),
		}
			.to_string()
	}
}

impl JniToolboxError for jni::errors::JniError {
	fn jclass(&self) -> String {
		match self {
			_ => "java/lang/RuntimeException",
			// jni::errors::JniError::Unknown => todo!(),
			// jni::errors::JniError::ThreadDetached => todo!(),
			// jni::errors::JniError::WrongVersion => todo!(),
			// jni::errors::JniError::NoMemory => todo!(),
			// jni::errors::JniError::AlreadyCreated => todo!(),
			// jni::errors::JniError::InvalidArguments => todo!(),
			// jni::errors::JniError::Other(_) => todo!(),
		}
			.to_string()
	}
}
