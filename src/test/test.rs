use jni_toolbox::jni;

#[jni(package = "toolbox", class = "Main")]
fn sum(a: i32, b: i32) -> i32 {
	a + b
}

#[jni(package = "toolbox", class = "Main")]
fn concat(a: String, b: String) -> String {
	format!("{a} -- {b}")
}
