package rustjni.test;

import java.nio.file.Path;
import java.nio.file.Paths;

public class ClassWithObjectNativeMethods {
  static {
    System.load(Paths.get("../../target/debug/deps/librust_jni_java_dylib.so").toAbsolutePath().toString());
  }

  public native SimpleClass testNativeFunction(SimpleClass input);

  public SimpleClass testFunction(SimpleClass input) {
    return testNativeFunction(input);
  }

  public static native SimpleClass testStaticNativeFunction(SimpleClass input);

  public static SimpleClass testStaticFunction(SimpleClass input) {
    return testStaticNativeFunction(input);
  }
}
