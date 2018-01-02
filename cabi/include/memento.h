/* begin generated memento header (see cabi/build.rs) */

#ifndef MEMENTO_H_INCLUDED
#define MEMENTO_H_INCLUDED

#include <stdint.h>
#include <stdlib.h>
#include <stdbool.h>

enum MementoErrorCode {
  NoError = 0,
  InvalidEncoding = 101,
  IoError = 1001,
  ParseEerror = 1002,
  InvalidTimeRange = 1003,
  InvalidTimeStart = 1004,
  InvalidTimeEnd = 1005,
  NoArchiveAvailable = 1006,
  CorruptDatabase = 1007,
};
typedef uint32_t MementoErrorCode;

typedef struct {
  MementoErrorCode error;
} MementoResult;

typedef struct {
  const char *data;
  size_t len;
  bool owned;
} MementoStr;

MementoResult memento_fetch_path(const MementoStr *path, uint64_t from, uint64_t until);

void memento_free_str(MementoStr *s);

MementoStr memento_new_str(char *c);

#endif /* MEMENTO_H_INCLUDED */

/* end generated memento header (see cabi/build.rs) */
