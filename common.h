#ifndef COMMON_H
#define COMMON_H

#ifndef MINQLXTISM_VERSION
#define MINQLXTISM_VERSION "VERSION NOT SET"
#endif

#define DEBUG_PRINT_PREFIX "[minqlxtism] "
#define DEBUG_ERROR_FORMAT "[minqlxtism] ERROR @ %s:%d in %s:\n" DEBUG_PRINT_PREFIX

#ifndef NOPY
#define SV_TAGS_PREFIX "minqlxtism"
#else
#define SV_TAGS_PREFIX "minqlxtism-nopy"
#endif

// TODO: Add minqlxtism version to serverinfo.

#include <stdint.h>

#include "maps_parser.h"

// We need an unsigned integer that's guaranteed to be the size of a pointer.
// "unsigned int" should do it, but we might want to port this to Windows for
// listen servers, where ints are 32 even on 64-bit Windows, so we're explicit.
#if defined(__x86_64__) || defined(_M_X64)
typedef uint64_t pint;
typedef int64_t sint;
#define __cdecl
#elif defined(__i386) || defined(_M_IX86)
typedef uint32_t pint;
typedef int32_t sint;
#define __cdecl __attribute__((__cdecl__))
#endif

extern int common_initialized;
extern int cvars_initialized;

void InitializeStatic(void);
void InitializeVm(void);
void InitializeCvars(void);
void SearchVmFunctions(void); // Needs to be called every time the VM is loaded.
void HookStatic(void);
void HookVm(void);
void DebugPrint(const char* fmt, ...);
void DebugError(const char* fmt, const char* file, int line, const char* func, ...);

// Misc.
int GetPendingPlayer(uint64_t* players);
void SetPendingPlayer(uint64_t* players, int client_id);
float RandomFloat(void);
float RandomFloatWithNegative(void);
void* PatternSearch(void* address, size_t length, const char* pattern, const char* mask);
void* PatternSearchModule(module_info_t* module, const char* pattern, const char* mask);

#endif /* COMMON_H */
