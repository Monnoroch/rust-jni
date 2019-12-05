package rustjni.test;

public class ClassWithPrimitiveMethods {
  public void testFunction() {}

  public boolean testFunction(boolean input) {
    return !input;
  }

  public char testFunction(char input) {
    return (char) (input + 1);
  }

  public byte testFunction(byte input) {
    return (byte) (input + 2);
  }

  public short testFunction(short input) {
    return (short) (input + 3);
  }

  public int testFunction(int input) {
    return input + 4;
  }

  public long testFunction(long input) {
    return input + 5;
  }

  // TODO(#25): floating point numbers don't work properly.
  public float testFloatFunction(double input) {
    return (float) input + 6;
  }

  public double testFunction(double input) {
    return input + 7;
  }

  public static void testStaticFunction() {}

  public static boolean testStaticFunction(boolean input) {
    return !input;
  }

  public static char testStaticFunction(char input) {
    return (char) (input + 1);
  }

  public static byte testStaticFunction(byte input) {
    return (byte) (input + 2);
  }

  public static short testStaticFunction(short input) {
    return (short) (input + 3);
  }

  public static int testStaticFunction(int input) {
    return input + 4;
  }

  public static long testStaticFunction(long input) {
    return input + 5;
  }

  // TODO(#25): floating point numbers don't work properly.
  public static float testStaticFloatFunction(double input) {
    return (float) input + 6;
  }

  public static double testStaticFunction(double input) {
    return input + 7;
  }
}
