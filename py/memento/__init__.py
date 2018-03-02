# -*- coding: utf-8 -*-

import time
from memento._native import ffi, lib


AGGREGATION_TYPES = {
    1: 'average',
    2: 'sum',
    3: 'last',
    4: 'max',
    5: 'min',
    6: 'avg_zero',
    7: 'absmax',
    8: 'absmin'
}

DEFAULT_AGGREGATION = AGGREGATION_TYPES[1]


def info(path):
    res = lib.memento_header_fetch(path.encode('utf-8'))

    try:
        if lib.memento_header_is_error(res):
            raise RuntimeError("Failed to read header, Error code {}".format(res.error))
        return _read_header(res.header)
    finally:
        lib.memento_header_free(res)


def _read_header(header):
    aggregation = AGGREGATION_TYPES.get(header.metadata.aggregation, DEFAULT_AGGREGATION)
    info = {
        'aggregationMethod': aggregation,
        'maxRetention': header.metadata.max_retention,
        'xFilesFactor': header.metadata.x_files_factor
    }

    archives = []
    num_archives = header.metadata.archive_count

    for i in range(num_archives):
        seconds_per_point = header.archives[i].seconds_per_point
        num_points = header.archives[i].num_points

        archives.append({
            'offset': header.archives[i].offset,
            'secondsPerPoint': seconds_per_point,
            'points': num_points,
            'retention': seconds_per_point * num_points,
            'size': num_points * 12
        })

    info['archives'] = archives
    return info


def fetch(path, from_time, until_time=None, now=None):
    pass
