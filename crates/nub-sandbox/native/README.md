# Windows native compatibility

The optional adapter runs inside an AppContainer process. AppContainer remains the enforcement boundary; the adapter supplies a real null-device handle, translates DOS volume aliases, and redirects supported runtime coordination objects into the package namespace. Child injection happens before the initial thread resumes.

The parent injection code is statically linked. Both x64 and ARM64 DLLs are built from source and embedded in the Rust library. Windows MSVC builds require the x64 and ARM64 C++ build tools and Windows SDK libraries. No compiler or separate injector executable is needed at runtime.

The `detours` directory contains Microsoft Detours sources from commit `adb07604aa56508448b95bf037c2a6d0d3b6831a`, under the included MIT license. Upstream: https://github.com/microsoft/Detours
