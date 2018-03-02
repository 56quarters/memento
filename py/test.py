# -*- coding: utf-8 -*-


import os
import sys

built_module = os.path.join(os.path.dirname(os.path.abspath(__file__)), 'build', 'lib')
sys.path.insert(0, built_module)
print("Import path:", sys.path)

import memento


print(memento.info('../tests/count_01.wsp'))
