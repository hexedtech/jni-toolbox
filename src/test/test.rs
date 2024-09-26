use jni_toolbox::{jni, JniToolboxError};

#[jni(package = "toolbox", class = "Main")]
fn sum(a: i32, b: i32) -> i32 {
	a + b
}

#[jni(package = "toolbox", class = "Main")]
fn concat(a: String, b: String) -> String {
	format!("{a} -- {b}")
}

#[jni(package = "toolbox", class = "Main")]
fn to_vec(a: String, b: String, c: String) -> Vec<String> {
	vec![a, b, c]
}

#[jni(package = "toolbox", class = "Main")]
fn maybe(idk: Option<String>) -> bool {
	idk.is_some()
}

#[jni(package = "toolbox", class = "Main")]
fn optional(present: bool) -> Option<String> {
	if present {
		Some("hello world!".into())
	} else {
		None
	}
}

#[jni(package = "toolbox", class = "Main")]
fn raw<'local>(env: &mut jni::JNIEnv<'local>) -> Result<jni::objects::JString<'local>, jni::errors::Error> {
	env.new_string("hello world!")
}

#[derive(thiserror::Error, Debug)]
#[error("some test error")]
struct CustomError;

impl JniToolboxError for CustomError {
	fn jclass(&self) -> String {
		"toolbox/CustomException".to_string()	
	}
}

#[jni(package = "toolbox", class = "Main")]
fn throw_error() -> Result<(), CustomError> {
	Err(CustomError)
}
