use jni_toolbox::jni;

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
