package toolbox;

import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertThrows;


public class Main {
	static {
		System.loadLibrary("jni_toolbox_test");
	}

	static native int sum(int a, int b);

	@Test
	public void argumentsByValue() {
		assertEquals(Main.sum(42, 13), 42 + 13);
	}

	static native String concat(String a, String b);

	@Test
	public void argumentsByReference() {
		assertEquals(Main.concat("hello", "world"), "hello -- world");
	}

	static native String[] to_vec(String a, String b, String c);

	@Test
	public void returnVec() {
		String[] actual = new String[]{"a", "b", "c"};
		String[] from_rust = Main.to_vec("a", "b", "c");
		for (int i = 0; i < 3; i++) {
			assertEquals(actual[i], from_rust[i]);
		}
	}

	static native boolean maybe(String optional);

	@Test
	public void optional() {
		assertEquals(Main.maybe(null), false);
		assertEquals(Main.maybe("aa"), true);
	}

	static native String raw();

	@Test
	public void passEnv() {
		assertEquals(Main.raw(), "hello world!");
	}

	static native String optional(boolean present);

	@Test
	public void nullableReturn() {
		assertNull(Main.optional(false));
		assertEquals(Main.optional(true), "hello world!");
	}

	static native void throw_error();

	@Test
	public void throwError() {
		assertThrows(CustomException.class, Main::throw_error);
	}

	static native CustomClass create_jclass(String field, boolean flag);

	@Test
	public void createDataClass() {
		CustomClass cc = create_jclass("hi from java", true);
		assertEquals(cc.field, "hi from java");
		assertEquals(cc.flag, true);
	}

	static native String receive_jclass(CustomClass input);

	@Test
	public void sendDataClass() {
		CustomClass cc = new CustomClass("hi from java", true);
		assertEquals(receive_jclass(cc), "hi from java");
	}

	static native SomeEnum create_enum();

	@Test
	public void tryCreateEnum() {
		assertEquals(create_enum(), SomeEnum.FIRST_VARIANT);
	}

	static native int enum_rust_value(SomeEnum input);

	@Test
	public void tryPassEnum() {
		assertEquals(enum_rust_value(SomeEnum.SECOND_VARIANT), 3);
	}
}
