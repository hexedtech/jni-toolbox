pub mod into_java;
pub mod from_java;
pub mod raw_java;

pub use jni_toolbox_macro::jni;
pub use into_java::{IntoJavaObject, IntoJavaPrimitive};
pub use from_java::{FromJava, from_java_static};
pub use raw_java::IntoJavaRaw;


/// An error that is meant to be used with jni-toolbox.
pub trait JniToolboxError: std::error::Error {
	/// The Java class for the matching exception.
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
