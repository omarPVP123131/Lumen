cmake_minimum_required(VERSION 3.22.1)
project("lumen_mobile_native")

# Compila el puente JNI nativo en una biblioteca compartida .so.
# El puente se genera con `lumen build --target android`, que deposita aqui el
# .c correspondiente; por eso no vive en el repositorio. Si falta, CMake avisa
# con un mensaje util en vez de fallar con "no sources given to target".
set(LUMEN_JNI_SRC "${CMAKE_CURRENT_SOURCE_DIR}/lumen_jni.c")
if(NOT EXISTS "${LUMEN_JNI_SRC}")
    message(FATAL_ERROR
        "Falta lumen_jni.c. Generalo con: lumen build --target android <programa>.nv")
endif()

add_library(lumen_native SHARED
    "${LUMEN_JNI_SRC}"
)

find_library(log-lib log)

target_link_libraries(lumen_native
    ${log-lib}
)
