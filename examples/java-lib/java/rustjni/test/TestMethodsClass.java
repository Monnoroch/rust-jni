package rustjni.test;

import java.nio.file.Path;
import java.nio.file.Paths;

public class TestMethodsClass {
  static {
    System.load(Paths.get("dylib/target/debug/librust_jni_java_dylib.so").toAbsolutePath().toString());
  }

  public native void testNativeFunction();
  public native boolean testNativeFunction(boolean input1, boolean input2, boolean input3);
  public native char testNativeFunction(char input1, char input2, char input3);
  public native byte testNativeFunction(byte input1, byte input2, byte input3);
  public native short testNativeFunction(short input1, short input2, short input3);
  public native int testNativeFunction(int input1, int input2, int input3);
  public native long testNativeFunction(long input1, long input2, long input3);
  public native float testNativeFunction(float input1, float input2, float input3);
  public native float testNativeFunctionF(double input1, double input2, double input3);
  public native double testNativeFunction(double input1, double input2, double input3);
  public native TestMethodsClass testNativeFunction(TestMethodsClass input1, TestMethodsClass input2, TestMethodsClass input3);
  public native TestInterface testNativeFunction(TestInterface input1, TestInterface input2, TestInterface input3);

  public void testFunction() {
    testNativeFunction();
  }

  public boolean testFunction(boolean input1, boolean input2, boolean input3) {
    return testNativeFunction(input1, input2, input3);
  }

  public char testFunction(char input1, char input2, char input3) {
    return Character.toString(
      testNativeFunction(
        Character.toString(input1).charAt(0),
        Character.toString(input2).charAt(0),
        Character.toString(input3).charAt(0)
      )
    ).charAt(0);
  }

  public byte testFunction(byte input1, byte input2, byte input3) {
    return testNativeFunction(input1, input2, input3);
  }

  public short testFunction(short input1, short input2, short input3) {
    return testNativeFunction(input1, input2, input3);
  }

  public int testFunction(int input1, int input2, int input3) {
    return testNativeFunction(input1, input2, input3);
  }

  public long testFunction(long input1, long input2, long input3) {
    return testNativeFunction(input1, input2, input3);
  }

  public float testFunction(float input1, float input2, float input3) {
    return testNativeFunction(input1, input2, input3);
  }

  public double testFunction(double input1, double input2, double input3) {
    return testNativeFunction(input1, input2, input3);
  }

  public TestMethodsClass testFunction(TestMethodsClass input1, TestMethodsClass input2, TestMethodsClass input3) {
    return testNativeFunction(input1, input2, input3);
  }

  public TestInterface testFunction(TestInterface input1, TestInterface input2, TestInterface input3) {
    return testNativeFunction(input1, input2, input3);
  }

  public static native void testStaticNativeFunction();
  public static native boolean testStaticNativeFunction(boolean input1, boolean input2, boolean input3);
  public static native char testStaticNativeFunction(char input1, char input2, char input3);
  public static native byte testStaticNativeFunction(byte input1, byte input2, byte input3);
  public static native short testStaticNativeFunction(short input1, short input2, short input3);
  public static native int testStaticNativeFunction(int input1, int input2, int input3);
  public static native long testStaticNativeFunction(long input1, long input2, long input3);
  public static native float testStaticNativeFunction(float input1, float input2, float input3);
  public static native double testStaticNativeFunction(double input1, double input2, double input3);
  public static native TestMethodsClass testStaticNativeFunction(TestMethodsClass input1, TestMethodsClass input2, TestMethodsClass input3);
  public static native TestInterface testStaticNativeFunction(TestInterface input1, TestInterface input2, TestInterface input3);

  public static void testStaticFunction() {
    testStaticNativeFunction();
  }

  public static boolean testStaticFunction(boolean input1, boolean input2, boolean input3) {
    return testStaticNativeFunction(input1, input2, input3);
  }

  public static char testStaticFunction(char input1, char input2, char input3) {
    return Character.toString(
      testStaticNativeFunction(
        Character.toString(input1).charAt(0),
        Character.toString(input2).charAt(0),
        Character.toString(input3).charAt(0)
      )
    ).charAt(0);
  }

  public static byte testStaticFunction(byte input1, byte input2, byte input3) {
    return testStaticNativeFunction(input1, input2, input3);
  }

  public static short testStaticFunction(short input1, short input2, short input3) {
    return testStaticNativeFunction(input1, input2, input3);
  }

  public static int testStaticFunction(int input1, int input2, int input3) {
    return testStaticNativeFunction(input1, input2, input3);
  }

  public static long testStaticFunction(long input1, long input2, long input3) {
    return testStaticNativeFunction(input1, input2, input3);
  }

  public static float testStaticFunction(float input1, float input2, float input3) {
    return testStaticNativeFunction(input1, input2, input3);
  }

  public static double testStaticFunction(double input1, double input2, double input3) {
    return testStaticNativeFunction(input1, input2, input3);
  }

  public static TestMethodsClass testStaticFunction(TestMethodsClass input1, TestMethodsClass input2, TestMethodsClass input3) {
    return testStaticNativeFunction(input1, input2, input3);
  }

  public static TestInterface testStaticFunction(TestInterface input1, TestInterface input2, TestInterface input3) {
    return testStaticNativeFunction(input1, input2, input3);
  }
}
