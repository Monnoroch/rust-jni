package rustjni.test;

import java.nio.file.Path;
import java.nio.file.Paths;

public class TestMethodsClass {
  public void testFunction() {
    testStaticFunction();
  }

  public boolean testFunction(boolean input1, boolean input2, boolean input3) {
    return testStaticFunction(input1, input2, input3);
  }

  public char testFunction(char input1, char input2, char input3) {
    return Character.toString(
      testStaticFunction(
        Character.toString(input1).charAt(0),
        Character.toString(input2).charAt(0),
        Character.toString(input3).charAt(0)
      )
    ).charAt(0);
  }

  public byte testFunction(byte input1, byte input2, byte input3) {
    return testStaticFunction(input1, input2, input3);
  }

  public short testFunction(short input1, short input2, short input3) {
    return testStaticFunction(input1, input2, input3);
  }

  public int testFunction(int input1, int input2, int input3) {
    return testStaticFunction(input1, input2, input3);
  }

  public long testFunction(long input1, long input2, long input3) {
    return testStaticFunction(input1, input2, input3);
  }

  public float testFunction(float input1, float input2, float input3) {
    return testStaticFunction(input1, input2, input3);
  }

  public double testFunction(double input1, double input2, double input3) {
    return testStaticFunction(input1, input2, input3);
  }

  public TestMethodsClass testFunction(TestMethodsClass input1, TestMethodsClass input2, TestMethodsClass input3) {
    return testStaticFunction(input1, input2, input3);
  }

  public static void testStaticFunction() {
  }

  public static boolean testStaticFunction(boolean input1, boolean input2, boolean input3) {
    return input2;
  }

  public static char testStaticFunction(char input1, char input2, char input3) {
    return Character.toString(input2).charAt(0);
  }

  public static byte testStaticFunction(byte input1, byte input2, byte input3) {
    return input2;
  }

  public static short testStaticFunction(short input1, short input2, short input3) {
    return input2;
  }

  public static int testStaticFunction(int input1, int input2, int input3) {
    return input2;
  }

  public static long testStaticFunction(long input1, long input2, long input3) {
    return input2;
  }

  public static float testStaticFunction(float input1, float input2, float input3) {
    return input2;
  }

  public static double testStaticFunction(double input1, double input2, double input3) {
    return input2;
  }

  public static TestMethodsClass testStaticFunction(TestMethodsClass input1, TestMethodsClass input2, TestMethodsClass input3) {
    return input2;
  }
}
