"""
An example world for the component to target.
"""
from typing import TypeVar, Generic, Union, Optional, Protocol, Tuple, List, Any, Self
from types import TracebackType
from enum import Flag, Enum, auto
from dataclasses import dataclass
from abc import abstractmethod
import weakref

from .types import Result, Ok, Err, Some
from .imports import types


class Guest(Protocol):

    @abstractmethod
    def hello_world(self) -> str:
        """
        This exported function can't be called automatically from Wasvy
        because it doesn't comply to the desired signature.
        """
        raise NotImplementedError

    @abstractmethod
    def print_first_component_system(self, params: List[List[types.QueryResultEntry]]) -> None:
        """
        All systems must only have one argument of type `list<query-result>`
        """
        raise NotImplementedError

    @abstractmethod
    def setup(self) -> None:
        """
        This function is called once on startup for each WASM component (Not Bevy component).
        """
        raise NotImplementedError

