use jni_toolbox::{jclass, jni, jenum};

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
fn raw<'local>(env: &mut jni::Env<'local>) -> Result<jni::objects::JString<'local>, jni::errors::Error> {
	env.new_string("hello world!")
}

#[derive(thiserror::Error, Debug)]
#[error("some test error")]
struct CustomError(i32);

impl jni_toolbox::IntoException for CustomError {
	fn jclass(&self) -> &'static str {
		"toolbox/CustomException"
	}
}

#[jni(package = "toolbox", class = "Main")]
fn throw_error() -> Result<(), CustomError> {
	Err(CustomError(42))
}

#[jclass(package = "toolbox")] // class name inferred from the struct
struct CustomClass {
	field: String,
	flag: bool,
}

#[jni(package = "toolbox", class = "Main")]
fn create_jclass(field: String, flag: bool) -> CustomClass {
	CustomClass { field, flag }
}

#[jni(package = "toolbox", class = "Main")]
fn receive_jclass(input: CustomClass) -> String {
	input.field.clone()
}

#[jenum(package = "toolbox")]
enum SomeEnum {
	FirstVariant = 2,
	SecondVariant = 3
}

#[jni(package = "toolbox", class = "Main")]
fn create_enum() -> SomeEnum {
	SomeEnum::FirstVariant
}

#[jni(package = "toolbox", class = "Main")]
fn enum_rust_value(input: SomeEnum) -> i32 {
	input as i32
}
