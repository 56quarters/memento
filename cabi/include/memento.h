/* generated memento header (see cabi/build.rs) */

#ifndef MEMENTO_H_INCLUDED
#define MEMENTO_H_INCLUDED

#include <stdint.h>
#include <stdlib.h>
#include <stdbool.h>

enum MementoErrorCode {
  NoError = 0,
  InvalidString = 101,
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
  double value;
  uint32_t timestamp;
} MementoPoint;

typedef struct {
  MementoPoint *points;
  size_t size;
  MementoErrorCode error;
} MementoResult;

/*
 *
 *
 *
 */
MementoResult *memento_result_fetch(const char *path, uint64_t from, uint64_t until);

/*
 *
 *
 *
 */
void memento_result_free(MementoResult *res);

/*
 *
 *
 *
 */
bool memento_result_is_error(const MementoResult *res);

#endif /* MEMENTO_H_INCLUDED */
