# -*- coding: utf-8 -*-

import timeit

print('Memento header read', timeit.timeit(
    stmt='memento.info("../tests/count_01.wsp")',
    setup="import memento"
))

print('Whisper header read', timeit.timeit(
    stmt='whisper.info("../tests/count_01.wsp")',
    setup="import whisper"
))

"""
print('Memento points read', timeit.timeit(
    stmt='memento.fetch("../tests/upper_01.wsp", 1502089980, 1502259660, 1502864800)',
    setup="import memento"
))

print('Whisper points read', timeit.timeit(
    stmt='whisper.fetch("../tests/upper_01.wsp", 1502089980, 1502259660, 1502864800)',
    setup="import whisper"
))
"""
