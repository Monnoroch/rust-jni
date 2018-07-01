package rustjni.test;

public final class TestClassExtendedFinal extends TestClassExtended {
  public long testClassExtendedFinalFunction(int input) {
  	return input;
  }

  public static TestClassExtendedFinal create() {
    return new TestClassExtendedFinal();
  }
}
