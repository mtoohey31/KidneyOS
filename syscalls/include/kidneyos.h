/*
 * KidneyOS Syscalls
 *
 * This header contains stubs for all the different syscalls that you can use in your C programs.
 * This file is automatically generated by the kidneyos-syscalls crate.
 */

#ifndef KIDNEYOS_SYSCALLS_H
#define KIDNEYOS_SYSCALLS_H

#include <stdint.h>

typedef uint16_t Pid;

typedef struct Timespec {

} Timespec;

void exit(uintptr_t code);

Pid fork(void);

uintptr_t read(uint32_t fd, uint8_t *buffer, uintptr_t count);

Pid waitpid(Pid pid, int32_t *stat, int32_t options);

void execve(const uint8_t *elf_bytes, uintptr_t byte_count);

int32_t nanosleep(const struct Timespec *duration, struct Timespec *remainder);

int32_t scheduler_yield(void);

#endif  /* KIDNEYOS_SYSCALLS_H */
