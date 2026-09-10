"""Opt-in AppContainer support for Python's private directory creation.

Execute from an embedder-owned sitecustomize module. This is compatibility code,
not the security boundary; omitting it leaves the OS policy in force.
"""


def _install():
    import os
    if os.name != "nt" or getattr(os.mkdir, "_appcontainer_compatible", False):
        return

    import ctypes
    from ctypes import wintypes
    import sys

    kernel = ctypes.WinDLL("kernel32", use_last_error=True)
    advapi = ctypes.WinDLL("advapi32", use_last_error=True)
    kernel.GetCurrentProcess.restype = wintypes.HANDLE
    kernel.CloseHandle.argtypes = [wintypes.HANDLE]
    kernel.LocalFree.argtypes = [ctypes.c_void_p]
    kernel.LocalFree.restype = ctypes.c_void_p
    advapi.OpenProcessToken.argtypes = [wintypes.HANDLE, wintypes.DWORD,
                                        ctypes.POINTER(wintypes.HANDLE)]
    advapi.GetTokenInformation.argtypes = [wintypes.HANDLE, ctypes.c_int,
                                           ctypes.c_void_p, wintypes.DWORD,
                                           ctypes.POINTER(wintypes.DWORD)]
    advapi.ConvertSidToStringSidW.argtypes = [ctypes.c_void_p,
                                            ctypes.POINTER(wintypes.LPWSTR)]
    advapi.ConvertStringSecurityDescriptorToSecurityDescriptorW.argtypes = [
        wintypes.LPCWSTR, wintypes.DWORD, ctypes.POINTER(ctypes.c_void_p),
        ctypes.POINTER(wintypes.DWORD)]

    token = wintypes.HANDLE()
    if not advapi.OpenProcessToken(kernel.GetCurrentProcess(), 8, ctypes.byref(token)):
        raise ctypes.WinError(ctypes.get_last_error())
    try:
        size = wintypes.DWORD()
        advapi.GetTokenInformation(token, 31, None, 0, ctypes.byref(size))
        if not size.value:
            raise ctypes.WinError(ctypes.get_last_error())
        info = ctypes.create_string_buffer(size.value)
        if not advapi.GetTokenInformation(token, 31, info, size, ctypes.byref(size)):
            raise ctypes.WinError(ctypes.get_last_error())
        sid = ctypes.cast(info, ctypes.POINTER(ctypes.c_void_p))[0]
        if not sid:
            return  # Ordinary Python must retain its original directory behavior.
        text = wintypes.LPWSTR()
        if not advapi.ConvertSidToStringSidW(sid, ctypes.byref(text)):
            raise ctypes.WinError(ctypes.get_last_error())
        try:
            package_sid = text.value
        finally:
            kernel.LocalFree(text)
    finally:
        kernel.CloseHandle(token)

    class SecurityAttributes(ctypes.Structure):
        _fields_ = [("length", wintypes.DWORD), ("descriptor", ctypes.c_void_p),
                    ("inherit", wintypes.BOOL)]

    kernel.CreateDirectoryW.argtypes = [wintypes.LPCWSTR,
                                        ctypes.POINTER(SecurityAttributes)]
    original = os.mkdir

    def mkdir(path, mode=0o777, *, dir_fd=None):
        import operator
        mode = operator.index(mode)
        if mode != 0o700 or dir_fd is not None:
            return original(path, mode, dir_fd=dir_fd)
        path = os.fspath(path)
        decoded = os.fsdecode(path)
        if "\0" in decoded:
            raise ValueError("embedded null character")
        sys.audit("os.mkdir", path, mode, -1)
        # Preserve CPython's protected owner/admin/system ACL. Add only this
        # process's package SID, never All Application Packages or inherited ACEs.
        sddl = "D:P(A;OICI;FA;;;SY)(A;OICI;FA;;;BA)(A;OICI;FA;;;OW)"
        sddl += f"(A;OICI;FA;;;{package_sid})"
        descriptor = ctypes.c_void_p()
        if not advapi.ConvertStringSecurityDescriptorToSecurityDescriptorW(
                sddl, 1, ctypes.byref(descriptor), None):
            raise ctypes.WinError(ctypes.get_last_error())
        try:
            attributes = SecurityAttributes(ctypes.sizeof(SecurityAttributes), descriptor, False)
            if not kernel.CreateDirectoryW(decoded, ctypes.byref(attributes)):
                error = ctypes.get_last_error()
                raise OSError(error, ctypes.FormatError(error), path, error)
        finally:
            kernel.LocalFree(descriptor)

    mkdir._appcontainer_compatible = True
    os.mkdir = mkdir


_install()
del _install
