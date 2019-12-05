package rustjni.test;

import java.nio.file.Path;
import java.nio.file.Paths;

public class ClassWithPrimitiveNativeMethods {
  static {
    System.load(Paths.get("../../target/debug/deps/librust_jni_java_dylib.so").toAbsolutePath().toString());
  }

  public native void testNativeFunction();
  public native boolean testNativeFunction(boolean input);
  public native char testNativeFunction(char input);
  public native byte testNativeFunction(byte input);
  public native short testNativeFunction(short input);
  public native int testNativeFunction(int input);
  public native long testNativeFunction(long input);
  public native float testNativeFunction(float input);
  public native double testNativeFunction(double input);

  public void testFunction() {
    testNativeFunction();
  }

  public boolean testFunction(boolean input) {
    return testNativeFunction(input);
  }

  public char testFunction(char input) {
    return Character.toString(
      testNativeFunction(
        Character.toString(input).charAt(0)
      )
    ).charAt(0);
  }

  public byte testFunction(byte input) {
    return testNativeFunction(input);
  }

  public short testFunction(short input) {
    return testNativeFunction(input);
  }

  public int testFunction(int input) {
    return testNativeFunction(input);
  }

  public long testFunction(long input) {
    return testNativeFunction(input);
  }

  public float testFloatFunction(double input) {
    return testNativeFunction((float) input);
  }

  public double testFunction(double input) {
    return testNativeFunction(input);
  }

  public static native void testStaticNativeFunction();
  public static native boolean testStaticNativeFunction(boolean input);
  public static native char testStaticNativeFunction(char input);
  public static native byte testStaticNativeFunction(byte input);
  public static native short testStaticNativeFunction(short input);
  public static native int testStaticNativeFunction(int input);
  public static native long testStaticNativeFunction(long input);
  public static native float testStaticNativeFunction(float input);
  public static native double testStaticNativeFunction(double input);

  public static void testStaticFunction() {
    testStaticNativeFunction();
  }

  public static boolean testStaticFunction(boolean input) {
    return testStaticNativeFunction(input);
  }

  public static char testStaticFunction(char input) {
    return Character.toString(
      testStaticNativeFunction(
        Character.toString(input).charAt(0)
      )
    ).charAt(0);
  }

  public static byte testStaticFunction(byte input) {
    return testStaticNativeFunction(input);
  }

  public static short testStaticFunction(short input) {
    return testStaticNativeFunction(input);
  }

  public static int testStaticFunction(int input) {
    return testStaticNativeFunction(input);
  }

  public static long testStaticFunction(long input) {
    return testStaticNativeFunction(input);
  }

  public static float testStaticFloatFunction(double input) {
    return testStaticNativeFunction((float) input);
  }

  public static double testStaticFunction(double input) {
    return testStaticNativeFunction(input);
  }
}
