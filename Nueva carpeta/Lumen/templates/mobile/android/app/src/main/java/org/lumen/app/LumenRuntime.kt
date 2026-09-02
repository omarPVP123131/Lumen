package org.lumen.app

object LumenRuntime {
    init {
        System.loadLibrary("lumen_native")
    }

    external fun evalSource(code: String): String
}
