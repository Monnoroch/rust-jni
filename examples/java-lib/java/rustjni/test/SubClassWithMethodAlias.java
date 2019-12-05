package rustjni.test;

public class SubClassWithMethodAlias extends SimpleClass {
  public SubClassWithMethodAlias(int value) {
    super(value + 1);
  }

  protected SubClassWithMethodAlias(int value, boolean raw) {
    super(value);
  }

  public SubClassWithMethodAlias combine(SubClassWithMethodAlias other) {
    return new SubClassWithMethodAlias(this.value + other.value * 2, true);
  }
}
