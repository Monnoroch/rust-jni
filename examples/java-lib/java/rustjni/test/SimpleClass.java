package rustjni.test;

public class SimpleClass {
  protected final int value;

  public SimpleClass(int value) {
    this.value = value;
  }

  public int valueWithAdded(int toAdd) {
    return this.value + toAdd;
  }

  public SimpleClass combine(SimpleClass other) {
    return new SimpleClass(this.value + other.value);
  }
}
