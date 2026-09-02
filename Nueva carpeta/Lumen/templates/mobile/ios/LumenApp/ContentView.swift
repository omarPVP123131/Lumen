import SwiftUI

struct ContentView: View {
    @State private var output: String = "Toca el botón para evaluar código LÚMEN"

    var body: some View {
        VStack(spacing: 20) {
            Text("🚀 LÚMEN Native on iOS (SwiftUI)")
                .font(.title2)
                .bold()

            Button(action: {
                let code = "funcion entero cuadrado(entero x) { retornar x * x; }"
                output = LumenBridge.shared.eval(code: code)
            }) {
                Label("Ejecutar en LÚMEN", systemImage: "bolt.fill")
                    .padding()
                    .frame(maxWidth: .infinity)
                    .background(Color.blue)
                    .foregroundColor(.white)
                    .cornerRadius(10)
            }

            ScrollView {
                Text(output)
                    .font(.system(.body, design: .monospaced))
                    .padding()
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .background(Color(.systemGray6))
                    .cornerRadius(8)
            }
        }
        .padding()
    }
}
