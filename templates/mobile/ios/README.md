# Plantilla de Integración Móvil LÚMEN para iOS / Swift (Xcode + C-ABI)

Esta plantilla permite integrar el compilador nativo de LÚMEN en aplicaciones de iOS y iPadOS usando Swift y SwiftUI.

## Compilación Cruzada del Motor LÚMEN para iOS
```powershell
# En Windows PowerShell o terminal
lumen build --native --target aarch64-apple-ios src/main.nv -o liblumen_aarch64.a
```

## Integración en Xcode
1. Arrastra `liblumen_aarch64.a` y `lumen_ios.h` a tu proyecto de Xcode.
2. Agrega `lumen_ios.h` en tu bridging header (`Bridging-Header.h`).
3. Invoca `LumenBridge.shared.eval(code: "...")` desde cualquier vista SwiftUI o UIKit.
