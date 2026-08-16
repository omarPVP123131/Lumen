# Plantilla de Integración Móvil LÚMEN para Android (Kotlin + NDK C/JNI)

Esta plantilla permite integrar el compilador y motor nativo de LÚMEN en aplicaciones de Android (Android Studio / Gradle).

## Requisitos
- Android Studio Iguana / Jellyfish o superior
- Android NDK (r25c o superior)
- CMake 3.22.1+

## Compilación Cruzada del Motor LÚMEN para Android
Para generar la biblioteca estática/dinámica nativa:
```powershell
# En Windows PowerShell
lumen build --native --target aarch64-linux-android src/main.nv -o liblumen_core.so
```

## Ejecución
1. Abre esta carpeta en Android Studio.
2. Sincroniza Gradle (`Sync Project with Gradle Files`).
3. Ejecuta en tu emulador o dispositivo físico Android (ARM64 / x86_64).
