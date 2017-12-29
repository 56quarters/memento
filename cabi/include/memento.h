/* begin generated memento header (see cabi/build.rs) */

#ifndef MEMENTO_H_INCLUDED
#define MEMENTO_H_INCLUDED

#include <stdint.h>
#include <stdlib.h>
#include <stdbool.h>

enum MementoErrorCode {
  NoError = 0,
  IoError = 1,
  ParseEerror = 3,
  InvalidTimeRange = 4,
  InvalidTimeStart = 5,
  InvalidTimeEnd = 6,
  NoArchiveAvailable = 7,
  CorruptDatabase = 8,
};
typedef uint32_t MementoErrorCode;

typedef struct {
  MementoErrorCode error;
} MementoResult;

MementoResult whisper_fetch_path(uint64_t from, uint64_t until);

#endif /* MEMENTO_H_INCLUDED */

/* end generated memento header (see cabi/build.rs) */
