# What JNA reads by name, kept as it is in an app that shrinks its code: JNA's
# own classes, which its native library calls back into, and the generated
# code, whose structures JNA reads field by field and whose native methods it
# binds to the library's symbols. JNA refers to AWT, which Android lacks.
-dontwarn java.awt.**
-keep class com.sun.jna.** { *; }
-keep class eranos.internal.** { *; }
