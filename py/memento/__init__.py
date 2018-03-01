# -*- coding: utf-8 -*-

import time
from memento._native import ffi, lib

def fetch():
    return lib.memento_header_fetch('../../tests/count_01.wsp')
