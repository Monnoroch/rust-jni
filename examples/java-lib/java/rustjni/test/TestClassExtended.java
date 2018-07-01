package rustjni.test;

public class TestClassExtended extends TestClass implements TestInterfaceExtended {
  public long testClassExtendedFunction(int input) {
    return input;
  }

  public long testInterfaceExtendedFunction(int input) {
    return testClassExtendedFunction(input);
  }

  public static TestClassExtended create() {
    return new TestClassExtended();
  }

  public static TestInterfaceExtended createInterface() {
    return new TestClassExtended();
  }
}
