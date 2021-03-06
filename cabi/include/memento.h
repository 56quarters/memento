/* Generated Memento header */

#ifndef MEMENTO_H_INCLUDED
#define MEMENTO_H_INCLUDED

#include <stdint.h>
#include <stdlib.h>
#include <stdbool.h>

enum AggregationType {
    Average = 1,
    Sum = 2,
    Last = 3,
    Max = 4,
    Min = 5,
    AvgZero = 6,
    AbsMax = 7,
    AbsMin = 8,
};
typedef uint32_t AggregationType;

enum MementoErrorCode {
    NoError = 0,
    InvalidString = 101,
    IoError = 1001,
    ParseError = 1002,
    InvalidTimeRange = 1003,
    InvalidTimeStart = 1004,
    InvalidTimeEnd = 1005,
    NoArchiveAvailable = 1006,
    CorruptDatabase = 1007,
};
typedef uint32_t MementoErrorCode;

typedef struct {
    AggregationType aggregation;
    uint32_t max_retention;
    float x_files_factor;
    uint32_t archive_count;
} MementoMetadata;

typedef struct {
    uint32_t offset;
    uint32_t seconds_per_point;
    uint32_t num_points;
} MementoArchiveInfo;

typedef struct {
    MementoMetadata metadata;
    MementoArchiveInfo *archives;
    size_t size;
} MementoHeader;

typedef struct {
    MementoHeader *header;
    MementoErrorCode error;
} MementoHeaderResult;

typedef struct {
    double value;
    uint32_t timestamp;
} MementoPoint;

typedef struct {
    MementoPoint *points;
    size_t size;
    MementoErrorCode error;
} MementoPointsResult;

/*
 * Fetch the header of a Whisper database file.
 *
 * The returned pointer will never be null. Callers must check the return
 * values with the `memento_header_is_error` function before trying to use
 * the pointer to the header contained in the result object. If the response
 * was unsuccessful, `header` will be null and `error` will contain an error
 * code indicating what went wrong.
 *
 * The result must be freed by calling `memento_header_free` for both
 * successful responses and error responses.
 *
 * This method will panic if the given path pointer is null.
 */
MementoHeaderResult *memento_header_fetch(const char *path);

/*
 * Free memory used by this result and any header associated with it.
 * This method will panic if the given result pointer is null.
 */
void memento_header_free(MementoHeaderResult *res);

/*
 * Return true if this result is an error, false otherwise. This
 * method will panic if the given result pointer is null.
 */
bool memento_header_is_error(const MementoHeaderResult *res);

/*
 * Fetch points contained in a Whisper database file between the
 * given start and end times (unix timestamps in seconds).
 *
 * The returned pointer will never be null. Callers must check the
 * return value with the `memento_result_is_error` function before
 * trying to use the array of points associated with it. If the response
 * was successful, `points` will be a pointer to the start of an array
 * of points and `size` will be the length of the array. If the response
 * was unsucessful, `points` will be null and `error` will contain an
 * error code indicating what went wrong.
 *
 * The result must be freed by calling `memento_points_free` for both
 * successful responses and error responses.
 *
 * This method will panic if the given path pointer is null.
 */
MementoPointsResult *memento_points_fetch(const char *path, int64_t from, int64_t until);

/*
 * Fetch points contained in a Whisper database file between the
 * given start and end times (unix timestamps in seconds) using the
 * given `now` time to determine if the request can be satisfied.
 *
 * The returned pointer will never be null. Callers must check the
 * return value with the `memento_result_is_error` function before
 * trying to use the array of points associated with it. If the response
 * was successful, `points` will be a pointer to the start of an array
 * of points and `size` will be the length of the array. If the response
 * was unsucessful, `points` will be null and `error` will contain an
 * error code indicating what went wrong.
 *
 * The result must be freed by calling `memento_points_free` for both
 * successful responses and error responses.
 *
 * This method will panic if the given path pointer is null.
 */
MementoPointsResult *memento_points_fetch_full(const char *path,
                                               int64_t from,
                                               int64_t until,
                                               int64_t now);

/*
 * Free memory used by this result and potentially any points associated
 * with it. This method will panic if the given result pointer is null.
 */
void memento_points_free(MementoPointsResult *res);

/*
 * Return true if this result is an error, false otherwise. This
 * method will panic if the given result pointer is null.
 */
bool memento_points_is_error(const MementoPointsResult *res);

#endif /* MEMENTO_H_INCLUDED */
