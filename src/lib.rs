pub use jni_toolbox_macro::jni;

pub trait JniToolboxError: std::error::Error {
	fn jclass(&self) -> String;
}
