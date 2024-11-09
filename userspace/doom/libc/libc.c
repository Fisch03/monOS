#include <stdarg.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <sys/types.h>

void vsnprintf_impl(char *str, size_t size, const char *format, va_list ap);

int errno = 0;
FILE *stdin = (FILE *)0;
FILE *stdout = (FILE *)1;
FILE *stderr = (FILE *)2;

void unimplemented(const char *msg);

int mkdir(const char *path, mode_t mode) {
  // TODO

  return 0;
}

int strncasecmp(const char *s1, const char *s2, size_t n) {
  unimplemented("strncasecmp");
}

char *strdup(const char *s) { unimplemented("strdup"); }
char *strstr(const char *haystack, const char *needle) {
  unimplemented("strstr");
}
char *strchr(const char *s, int c) { unimplemented("strchr"); }
// char *strrchr(const char *s, int c) { unimplemented("strrchr"); }
int strncmp(const char *s1, const char *s2, size_t n) {
  unimplemented("strncmp");
}

int fprintf(FILE *restrict, const char *restrict, ...) {
  unimplemented("fprintf");
}
int snprintf(char *restrict s, size_t n, const char *restrict format, ...) {
  unimplemented("snprintf");
}

int rename(const char *old, const char *new) { unimplemented("rename"); }
int remove(const char *pathname) { unimplemented("remove"); }

// FILE *fopen(const char *restrict filename, const char *restrict mode) {
//   unimplemented("fopen");
// }
// int fclose(FILE *) { unimplemented("fclose"); }
size_t fread(void *restrict ptr, size_t size, size_t nmemb,
             FILE *restrict stream) {
  unimplemented("fread");
}
// long ftell(FILE *stream) { unimplemented("ftell"); }
int fseek(FILE *stream, long offset, int whence) { unimplemented("fseek"); }

int sscanf(const char *restrict str, const char *restrict format, ...) {
  unimplemented("sscanf");
}

int atoi(const char *nptr) { unimplemented("atoi"); }
double atof(const char *nptr) { unimplemented("atof"); }
int abs(int j) { unimplemented("abs"); }

void exit(int status) { unimplemented("exit"); }
int system(const char *command) { unimplemented("system"); }

int isspace(int c) { unimplemented("isspace"); }
int toupper(int c) { unimplemented("toupper"); }

double fabs(double x) { unimplemented("fabs"); }
