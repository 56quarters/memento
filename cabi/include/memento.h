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

typedef struct Point Point;

typedef struct {
  MementoErrorCode error;
  Point *results;
  size_t size;
} MementoResult;

void mement_result_free(MementoResult *res);

MementoResult memento_fetch_path(const char *path, uint64_t from, uint64_t until);

bool memento_result_is_error(const MementoResult *res);

#endif /* MEMENTO_H_INCLUDED */

/* end generated memento header (see cabi/build.rs) */
