package com.example.home_storage_vulkan;

public class Rust {
    static {
        System.loadLibrary("main");
    }

    public static native void hi();
}
