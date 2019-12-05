package rustjni.test;

public class SubSubClassWithMethodOverride extends SubClassWithMethodOverride {
  public SubSubClassWithMethodOverride(int value) {
    super(value + 1);
  }

  @Override
  public int valueWithAdded(int toAdd) {
    return this.value + toAdd * 3;
  }
}
