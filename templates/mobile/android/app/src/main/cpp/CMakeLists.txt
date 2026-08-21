cmake_minimum_required(VERSION 3.22.1)
project("lumen_mobile_native")

# Compila el puente JNI nativo en una biblioteca compartida .so
add_library(lumen_native SHARED
    lumen_jni.c
)

find_library(log-lib log)

target_link_libraries(lumen_native
    ${log-lib}
)
