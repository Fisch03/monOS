#pragma once

// typedef char *va_list;
// #define va_start(ap, parmn) (void)((ap) = (char *)(&(parmn) + 1))
// #define va_end(ap) (void)((ap) = 0)
// #define va_arg(ap, type) (*(type *)va_arg_ptr(ap, sizeof(type)))
// #define va_arg_ptr(ap, size) (((ap) = ((ap) + size)) - size)

#define va_list __builtin_va_list
#define va_start(v, l) __builtin_va_start(v, l)
#define va_end(v) __builtin_va_end(v)
#define va_arg(v, l) __builtin_va_arg(v, l)
