from typing import TypeVar, Generic, Union, Optional, Protocol, Tuple, List, Any, Self
from types import TracebackType
from enum import Flag, Enum, auto
from dataclasses import dataclass
from abc import abstractmethod
import weakref

from ..types import Result, Ok, Err, Some
from ..imports import types


def register_system(name: str, queries: List[types.Query]) -> None:
    raise NotImplementedError

def register_component(path: str) -> int:
    raise NotImplementedError

def get_component_id(path: str) -> Optional[int]:
    raise NotImplementedError

def spawn(components: List[types.Component]) -> int:
    raise NotImplementedError

def this_function_does_nothing(entry: types.QueryResultEntry, query_result: List[types.QueryResultEntry]) -> None:
    """
    For some reason if the type isn't being used by a function, cargo component doesn't generate a binding for it.
    so this function is only to accumulate types so they are generated.
    """
    raise NotImplementedError

