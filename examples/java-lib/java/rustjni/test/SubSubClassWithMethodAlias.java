package rustjni.test;

public class SubSubClassWithMethodAlias extends SubClassWithMethodAlias {
  public SubSubClassWithMethodAlias(int value) {
    super(value + 1);
  }

  protected SubSubClassWithMethodAlias(int value, boolean raw) {
    super(value, raw);
  }

  public SubSubClassWithMethodAlias combine(SubSubClassWithMethodAlias other) {
    return new SubSubClassWithMethodAlias(this.value + other.value * 3, true);
  }
}
