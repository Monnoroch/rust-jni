package rustjni.test;

public class SubClassWithMethodOverride extends SimpleClass {
  public SubClassWithMethodOverride(int value) {
    super(value + 1);
  }

  @Override
  public int valueWithAdded(int toAdd) {
    return this.value + toAdd * 2;
  }
}
