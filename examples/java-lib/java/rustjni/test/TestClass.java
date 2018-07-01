package rustjni.test;

public class TestClass implements TestInterface {
  public long testClassFunction(int input) {
    return input;
  }

  public long testInterfaceFunction(int input) {
    return testClassFunction(input);
  }

  public static TestClass create() {
    return new TestClass();
  }
}
