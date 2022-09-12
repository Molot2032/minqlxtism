import _minqlxtended
import re as _re

__version__ = _minqlxtended.__version__

temp = _re.search("([0-9]+)\.([0-9]+)\.([0-9]+)", __version__)
try:
    __version_info__ = tuple(map(lambda i: int(temp.group(i)), [1, 2, 3]))
except:
    __version_info__ = (999, 999, 999)
del temp

# Put everything into a single module.
from _minqlxtended import *
from ._core import *
from ._plugin import *
from ._game import *
from ._events import *
from ._commands import *
from ._handlers import *
from ._player import *
from ._zmq import *
