# -*- coding: utf-8 -*-

import os
import timeit

built_module = os.path.join(os.path.dirname(os.path.abspath(__file__)), 'build', 'lib')

print('Memento header read', timeit.timeit(
    stmt='memento.info("../tests/count_01.wsp")',
    setup="""
import sys
sys.path.insert(0, '{include}')
import memento
    """.format(include=built_module)
))

print('Whisper header read', timeit.timeit(
    stmt='whisper.info("../tests/count_01.wsp")',
    setup="import whisper"
))
