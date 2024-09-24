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
	static native String concat(String a, String b);
	static native String[] to_vec(String a, String b, String c);
	static native boolean maybe(String optional);
	static native String optional(boolean present);
	static native String raw();

	@Test
	public void argumentsByValue() {
		assertEquals(Main.sum(42, 13), 42 + 13);
	}

	@Test
	public void argumentsByReference() {
		assertEquals(Main.concat("hello", "world"), "hello -- world");
	}

	@Test
	public void checksForNull() {
		// TODO maybe these should throw NullPtrException
		assertThrows(NullPointerException.class, () -> Main.concat("a", null));
		assertThrows(NullPointerException.class, () -> Main.concat(null, "a"));
		assertThrows(NullPointerException.class, () -> Main.concat(null, null));
	}

	@Test
	public void returnVec() {
		String[] actual = new String[]{"a", "b", "c"};
		String[] from_rust = Main.to_vec("a", "b", "c");
		for (int i = 0; i < 3; i++) {
			assertEquals(actual[i], from_rust[i]);
		}
	}

	@Test
	public void optional() {
		assertEquals(Main.maybe(null), false);
		assertEquals(Main.maybe("aa"), true);
	}

	@Test
	public void passEnv() {
		assertEquals(Main.raw(), "hello world!");
	}

	@Test
	public void nullableReturn() {
		assertNull(Main.optional(false));
		assertEquals(Main.optional(true), "hello world!");
	}
	
}
