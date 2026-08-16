package org.lumen.app

import android.os.Bundle
import android.widget.Button
import android.widget.TextView
import androidx.appcompat.app.AppCompatActivity

class MainActivity : AppCompatActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        
        val textView = TextView(this).apply {
            textSize = 16f
            setPadding(40, 60, 40, 40)
            text = "🚀 Presiona el botón para ejecutar código LÚMEN nativo en Android (AArch64)"
        }
        
        val button = Button(this).apply {
            text = "⚡ Ejecutar LÚMEN"
            setOnClickListener {
                val lumenCode = "funcion entero suma(entero a, entero b) { retornar a + b; }"
                val output = LumenRuntime.evalSource(lumenCode)
                textView.text = output
            }
        }

        val layout = android.widget.LinearLayout(this).apply {
            orientation = android.widget.LinearLayout.VERTICAL
            addView(button)
            addView(textView)
        }

        setContentView(layout)
    }
}
