pub use jni_toolbox_macro::jni;

pub trait JniToolboxError: std::error::Error {
	fn jclass(&self) -> String;
}

impl JniToolboxError for jni::errors::Error {
	fn jclass(&self) -> String {
		"java/lang/RuntimeException".to_string()
	}
}
