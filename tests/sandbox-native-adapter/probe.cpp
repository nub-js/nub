// Native compatibility feasibility probe. AppContainer remains the security boundary.
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <winternl.h>
#include <securityappcontainer.h>
#include <cstdio>
#include <cstdlib>
#include <cwchar>
#include <cstring>
#include <cstdint>
#include "detours.h"

static const GUID payload_id = {0x19c47458, 0xe2ad, 0x421d, {0x81, 0x37, 0x52, 0xa1, 0x85, 0xf7, 0xb8, 0x15}};
struct Payload {
    HANDLE null_device;
    char directory[MAX_PATH];
    wchar_t devices[26][MAX_PATH];
};
static Payload state = {};

static USHORT image_machine(HANDLE process) {
    // Suspended x64 processes on ARM64 can report native-machine via IsWow64Process2.
    // Read the mapped executable header, not the image path (which can be replaced).
    uintptr_t address = 0;
    MEMORY_BASIC_INFORMATION region;
    while (VirtualQueryEx(process, reinterpret_cast<void*>(address), &region, sizeof(region))) {
        auto base = static_cast<const BYTE*>(region.BaseAddress);
        if (region.Type == MEM_IMAGE && region.State == MEM_COMMIT &&
            region.AllocationBase == region.BaseAddress && !(region.Protect & (PAGE_GUARD | PAGE_NOACCESS))) {
            IMAGE_DOS_HEADER dos;
            IMAGE_NT_HEADERS32 pe;
            if (ReadProcessMemory(process, base, &dos, sizeof(dos), nullptr) &&
                dos.e_magic == IMAGE_DOS_SIGNATURE && dos.e_lfanew >= static_cast<LONG>(sizeof(dos)) &&
                static_cast<SIZE_T>(dos.e_lfanew) < region.RegionSize &&
                ReadProcessMemory(process, base + dos.e_lfanew, &pe, sizeof(pe), nullptr) &&
                pe.Signature == IMAGE_NT_SIGNATURE && !(pe.FileHeader.Characteristics & IMAGE_FILE_DLL)) {
                return pe.FileHeader.Machine;
            }
        }
        uintptr_t next = reinterpret_cast<uintptr_t>(base) + region.RegionSize;
        if (next <= address) break;
        address = next;
    }
    return 0;
}

static BOOL inject(HANDLE process, const Payload& source) {
    Payload target = source;
    USHORT reported = 0, native_machine = 0;
    IsWow64Process2(process, &reported, &native_machine);
    USHORT machine = image_machine(process);
    fprintf(stderr, "ADAPTER_MACHINE image=%04x wow=%04x native=%04x\n", machine, reported, native_machine);
    const char* arch = machine == IMAGE_FILE_MACHINE_AMD64 ? "x64" :
                       machine == IMAGE_FILE_MACHINE_ARM64 ? "arm64" : nullptr;
    if (!arch) { SetLastError(ERROR_EXE_MACHINE_TYPE_MISMATCH); return FALSE; }
    char path[MAX_PATH];
    if (sprintf_s(path, "%s\\probe-%s.dll", source.directory, arch) < 0) return FALSE;
    if (GetFileAttributesA(path) == INVALID_FILE_ATTRIBUTES) return FALSE;
    if (!DuplicateHandle(GetCurrentProcess(), source.null_device, process,
                         &target.null_device, 0, FALSE, DUPLICATE_SAME_ACCESS)) return FALSE;
    const char* dll = path;
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
                             state.directory, MAX_PATH, nullptr, &substituted) || substituted) return 5;
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
static auto true_create_file_a = CreateFileA;
static auto true_final_path = GetFinalPathNameByHandleW;
static auto true_create_process = CreateProcessW;
using NtDirectory = NTSTATUS (NTAPI*)(PHANDLE, ACCESS_MASK, POBJECT_ATTRIBUTES);
static NtDirectory true_create_directory = nullptr;
static NtDirectory true_open_directory = nullptr;

static bool msys_directory(POBJECT_ATTRIBUTES original, OBJECT_ATTRIBUTES& redirected,
                           UNICODE_STRING& name, wchar_t (&path)[1024]) {
    if (!original || original->RootDirectory || !original->ObjectName) return false;
    auto input = original->ObjectName;
    if (!input->Buffer || input->Length % sizeof(wchar_t) || input->Length >= 512 * sizeof(wchar_t)) return false;
    wchar_t source[512];
    memcpy(source, input->Buffer, input->Length);
    source[input->Length / sizeof(wchar_t)] = 0;
    const wchar_t* leaf = nullptr;
    const wchar_t global[] = L"\\BaseNamedObjects\\";
    const wchar_t session[] = L"\\Sessions\\BNOLINKS\\";
    if (!_wcsnicmp(source, global, wcslen(global))) leaf = source + wcslen(global);
    else if (!_wcsnicmp(source, session, wcslen(session))) {
        auto cursor = source + wcslen(session);
        auto start = cursor;
        while (*cursor >= L'0' && *cursor <= L'9') ++cursor;
        if (cursor != start && *cursor == L'\\') leaf = cursor + 1;
    }
    if (!leaf || (wcsncmp(leaf, L"msys-", 5) && wcsncmp(leaf, L"cygwin-", 7)) ||
        wcschr(leaf, L'\\') || wcschr(leaf, L'/')) return false;
    ULONG length = 0;
    if (!GetAppContainerNamedObjectPath(nullptr, nullptr, 1024, path, &length)) return false;
    if (path[0] != L'\\') {
        // The Win32 API returns a path relative to this session's BaseNamedObjects.
        wchar_t relative[1024];
        wcscpy_s(relative, path);
        DWORD session_id = 0;
        if (!ProcessIdToSessionId(GetCurrentProcessId(), &session_id)) return false;
        if (swprintf_s(path, L"\\Sessions\\%lu\\BaseNamedObjects\\%s", session_id, relative) < 0) return false;
    }
    size_t end = wcslen(path);
    if ((end && path[end - 1] != L'\\' && wcscat_s(path, L"\\")) || wcscat_s(path, leaf)) return false;
    name.Buffer = path;
    name.Length = USHORT(wcslen(path) * sizeof(wchar_t));
    name.MaximumLength = USHORT(name.Length + sizeof(wchar_t));
    redirected = *original;
    redirected.ObjectName = &name;
    // Keep shared runtime objects inside this package, using its token's default DACL.
    redirected.SecurityDescriptor = nullptr;
    return true;
}

static NTSTATUS NTAPI create_directory(PHANDLE handle, ACCESS_MASK access, POBJECT_ATTRIBUTES attrs) {
    OBJECT_ATTRIBUTES redirected;
    UNICODE_STRING name;
    wchar_t path[1024];
    if (!msys_directory(attrs, redirected, name, path)) return true_create_directory(handle, access, attrs);
    NTSTATUS status = true_create_directory(handle, access, &redirected);
    fprintf(stderr, "ADAPTER_MSYS_DIRECTORY path=%ls status=%08lx\n", path, static_cast<unsigned long>(status));
    return status;
}

static NTSTATUS NTAPI open_directory(PHANDLE handle, ACCESS_MASK access, POBJECT_ATTRIBUTES attrs) {
    OBJECT_ATTRIBUTES redirected;
    UNICODE_STRING name;
    wchar_t path[1024];
    return true_open_directory(handle, access,
                               msys_directory(attrs, redirected, name, path) ? &redirected : attrs);
}

static bool is_null(LPCWSTR path) {
    if (!path) return false;
    // PHP expands its NUL descriptor to an absolute DOS path before opening it.
    // Let Windows classify reserved DOS device names rather than matching file suffixes.
    using DosDeviceName = ULONG (NTAPI*)(PCWSTR);
    auto address = GetProcAddress(GetModuleHandleW(L"ntdll.dll"), "RtlIsDosDeviceName_U");
    DosDeviceName classify;
    memcpy(&classify, &address, sizeof(address));
    ULONG part = classify ? classify(path) : 0;
    return part && LOWORD(part) == 3 * sizeof(wchar_t) &&
           !_wcsnicmp(path + HIWORD(part) / sizeof(wchar_t), L"NUL", 3);
}

static HANDLE WINAPI create_file(LPCWSTR path, DWORD access, DWORD share,
                                LPSECURITY_ATTRIBUTES security, DWORD disposition,
                                DWORD flags, HANDLE template_file) {
    HANDLE result = true_create_file(path, access, share, security, disposition, flags, template_file);
    if (result == INVALID_HANDLE_VALUE && path && (wcsstr(path, L"NUL") || wcsstr(path, L"nul"))) {
        DWORD error = GetLastError();
        fprintf(stderr, "ADAPTER_NUL path=%ls access=%08lx flags=%08lx error=%lu recognized=%d\n", path, access, flags, error, is_null(path));
        SetLastError(error);
    }
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

static HANDLE WINAPI create_file_a(LPCSTR path, DWORD access, DWORD share,
                                   LPSECURITY_ATTRIBUTES security, DWORD disposition,
                                   DWORD flags, HANDLE template_file) {
    HANDLE result = true_create_file_a(path, access, share, security, disposition, flags, template_file);
    if (result != INVALID_HANDLE_VALUE || GetLastError() != ERROR_ACCESS_DENIED || !path) return result;
    if (_stricmp(path, "NUL") && _stricmp(path, "NUL:") &&
        _stricmp(path, "\\\\.\\NUL") && _stricmp(path, "\\\\?\\NUL")) return result;
    return create_file(L"NUL", access, share, security, disposition, flags, template_file);
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
    auto create = GetProcAddress(GetModuleHandleW(L"ntdll.dll"), "NtCreateDirectoryObject");
    auto open = GetProcAddress(GetModuleHandleW(L"ntdll.dll"), "NtOpenDirectoryObject");
    if (!create || !open) return FALSE;
    static_assert(sizeof(create) == sizeof(true_create_directory));
    memcpy(&true_create_directory, &create, sizeof(create));
    memcpy(&true_open_directory, &open, sizeof(open));
    DetourTransactionBegin();
    DetourUpdateThread(GetCurrentThread());
    DetourAttach(reinterpret_cast<PVOID*>(&true_create_file), create_file);
    DetourAttach(reinterpret_cast<PVOID*>(&true_create_file_a), create_file_a);
    DetourAttach(reinterpret_cast<PVOID*>(&true_final_path), final_path);
    DetourAttach(reinterpret_cast<PVOID*>(&true_create_process), create_process);
    DetourAttach(reinterpret_cast<PVOID*>(&true_create_directory), create_directory);
    DetourAttach(reinterpret_cast<PVOID*>(&true_open_directory), open_directory);
    return DetourTransactionCommit() == NO_ERROR;
}
#endif
