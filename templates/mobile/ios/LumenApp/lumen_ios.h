#ifndef LUMEN_IOS_H
#define LUMEN_IOS_H

#ifdef __cplusplus
extern "C" {
#endif

// LÚMEN Native C-ABI Bridge for iOS / iPadOS / macOS (AArch64 Apple Silicon)
const char* lumen_ios_eval(const char* source_code);

#ifdef __cplusplus
}
#endif

#endif // LUMEN_IOS_H
