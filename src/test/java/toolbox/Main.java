package toolbox;

import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.assertEquals;


public class Main {
	static {
		System.loadLibrary("jni_toolbox_test");
	}

	static native int sum(int a, int b);
	static native String concat(String a, String b);

	@Test
	public void argumentsByValue() {
		assertEquals(Main.sum(42, 13), 42 + 13);
	}

	@Test
	public void argumentsByReference() {
		assertEquals(Main.concat("hello", "world"), "hello -- world");
	}
	
}
