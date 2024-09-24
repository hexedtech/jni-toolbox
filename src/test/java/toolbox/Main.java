package toolbox;

import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;


public class Main {
	static {
		System.loadLibrary("jni_toolbox_test");
	}

	static native int sum(int a, int b);
	static native String concat(String a, String b);
	static native String[] to_vec(String a, String b, String c);
	static native boolean maybe(String optional);

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
		assertThrows(RuntimeException.class, () -> Main.concat("a", null));
		assertThrows(RuntimeException.class, () -> Main.concat(null, "a"));
		assertThrows(RuntimeException.class, () -> Main.concat(null, null));
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
	
}
