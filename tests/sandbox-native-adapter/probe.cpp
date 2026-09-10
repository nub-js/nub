// Native compatibility feasibility probe. AppContainer remains the security boundary.
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <cstdio>
#include <cstdlib>
#include <cwchar>
#include "detours.h"

static const GUID payload_id = {0x19c47458, 0xe2ad, 0x421d, {0x81, 0x37, 0x52, 0xa1, 0x85, 0xf7, 0xb8, 0x15}};
struct Payload {
    HANDLE null_device;
    char dll[MAX_PATH];
    wchar_t devices[26][MAX_PATH];
};
static Payload state = {};

static BOOL inject(HANDLE process, const Payload& source) {
    Payload target = source;
    if (!DuplicateHandle(GetCurrentProcess(), source.null_device, process,
                         &target.null_device, 0, FALSE, DUPLICATE_SAME_ACCESS)) return FALSE;
    const char* dll = target.dll;
    return DetourCopyPayloadToProcess(process, payload_id, &target, sizeof(target)) &&
           DetourUpdateProcessWithDll(process, &dll, 1);
}

#ifdef PROBE_INJECTOR
int wmain(int argc, wchar_t** argv) {
    if (argc != 3) return 2;
    DWORD pid = wcstoul(argv[1], nullptr, 10);
    HANDLE process = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_OPERATION |
                                 PROCESS_VM_READ | PROCESS_VM_WRITE | PROCESS_DUP_HANDLE,
                                 FALSE, pid);
    if (!process) { fprintf(stderr, "OpenProcess: %lu\n", GetLastError()); return 3; }
    state.null_device = CreateFileW(L"NUL", GENERIC_READ | GENERIC_WRITE,
                                   FILE_SHARE_READ | FILE_SHARE_WRITE, nullptr,
                                   OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, nullptr);
    if (state.null_device == INVALID_HANDLE_VALUE) return 4;
    BOOL substituted = FALSE;
    if (!WideCharToMultiByte(CP_ACP, WC_NO_BEST_FIT_CHARS, argv[2], -1,
                             state.dll, MAX_PATH, nullptr, &substituted) || substituted) return 5;
    for (int i = 0; i < 26; ++i) {
        wchar_t drive[] = {wchar_t(L'A' + i), L':', 0};
        QueryDosDeviceW(drive, state.devices[i], MAX_PATH);
    }
    BOOL ok = inject(process, state);
    DWORD error = GetLastError();
    CloseHandle(state.null_device);
    CloseHandle(process);
    fprintf(stderr, "INJECTION %s error=%lu\n", ok ? "ok" : "failed", error);
    return ok ? 0 : 6;
}
#else
static auto true_create_file = CreateFileW;
static auto true_final_path = GetFinalPathNameByHandleW;
static auto true_create_process = CreateProcessW;

static bool is_null(LPCWSTR path) {
    return path && (!_wcsicmp(path, L"NUL") || !_wcsicmp(path, L"NUL:") ||
                    !_wcsicmp(path, L"\\\\.\\NUL") || !_wcsicmp(path, L"\\\\?\\NUL"));
}

static HANDLE WINAPI create_file(LPCWSTR path, DWORD access, DWORD share,
                                LPSECURITY_ATTRIBUTES security, DWORD disposition,
                                DWORD flags, HANDLE template_file) {
    HANDLE result = true_create_file(path, access, share, security, disposition, flags, template_file);
    if (result != INVALID_HANDLE_VALUE || GetLastError() != ERROR_ACCESS_DENIED ||
        !is_null(path) || (flags & FILE_FLAG_OVERLAPPED)) return result;
    // Duplicate a parent-opened real null device, never a regular-file approximation.
    HANDLE duplicate = INVALID_HANDLE_VALUE;
    if (!DuplicateHandle(GetCurrentProcess(), state.null_device, GetCurrentProcess(),
                         &duplicate, access, security && security->bInheritHandle, 0)) {
        return INVALID_HANDLE_VALUE;
    }
    return duplicate;
}

static DWORD WINAPI final_path(HANDLE file, LPWSTR buffer, DWORD size, DWORD flags) {
    DWORD result = true_final_path(file, buffer, size, flags);
    if (result || GetLastError() != ERROR_ACCESS_DENIED || (flags & 7) != VOLUME_NAME_DOS) return result;
    wchar_t native[32768];
    DWORD length = true_final_path(file, native, 32768, flags | VOLUME_NAME_NT);
    if (!length || length >= 32768) return 0;
    for (int i = 0; i < 26; ++i) {
        size_t prefix = wcslen(state.devices[i]);
        if (!prefix || prefix > length || _wcsnicmp(native, state.devices[i], prefix) ||
            (native[prefix] && native[prefix] != L'\\')) continue;
        // Preserve the normalized/opened suffix and use a host-resolved drive alias.
        DWORD needed = DWORD(6 + length - prefix);
        if (size <= needed) return needed + 1;
        const wchar_t start[] = {L'\\', L'\\', L'?', L'\\', wchar_t(L'A' + i), L':', 0};
        wcscpy_s(buffer, size, start);
        wcscat_s(buffer, size, native + prefix);
        return needed;
    }
    SetLastError(ERROR_ACCESS_DENIED);
    return 0;
}

static BOOL WINAPI create_process(LPCWSTR application, LPWSTR command,
                                  LPSECURITY_ATTRIBUTES process_attrs,
                                  LPSECURITY_ATTRIBUTES thread_attrs, BOOL inherit,
                                  DWORD flags, LPVOID environment, LPCWSTR cwd,
                                  LPSTARTUPINFOW startup, LPPROCESS_INFORMATION child) {
    if (!true_create_process(application, command, process_attrs, thread_attrs, inherit,
                             flags | CREATE_SUSPENDED, environment, cwd, startup, child)) return FALSE;
    if (!inject(child->hProcess, state)) {
        DWORD error = GetLastError();
        TerminateProcess(child->hProcess, 127);
        WaitForSingleObject(child->hProcess, INFINITE);
        CloseHandle(child->hThread);
        CloseHandle(child->hProcess);
        SetLastError(error);
        return FALSE;
    }
    if (!(flags & CREATE_SUSPENDED) && ResumeThread(child->hThread) == DWORD(-1)) {
        DWORD error = GetLastError();
        TerminateProcess(child->hProcess, 127);
        WaitForSingleObject(child->hProcess, INFINITE);
        CloseHandle(child->hThread);
        CloseHandle(child->hProcess);
        SetLastError(error);
        return FALSE;
    }
    return TRUE;
}

extern "C" __declspec(dllexport) void ProbeMarker() {}
BOOL WINAPI DllMain(HINSTANCE, DWORD reason, LPVOID) {
    if (DetourIsHelperProcess()) return TRUE;
    if (reason != DLL_PROCESS_ATTACH) return TRUE;
    DetourRestoreAfterWith();
    DWORD size = 0;
    auto payload = static_cast<Payload*>(DetourFindPayloadEx(payload_id, &size));
    if (!payload || size != sizeof(Payload)) return FALSE;
    state = *payload;
    DetourTransactionBegin();
    DetourUpdateThread(GetCurrentThread());
    DetourAttach(reinterpret_cast<PVOID*>(&true_create_file), create_file);
    DetourAttach(reinterpret_cast<PVOID*>(&true_final_path), final_path);
    DetourAttach(reinterpret_cast<PVOID*>(&true_create_process), create_process);
    return DetourTransactionCommit() == NO_ERROR;
}
#endif
