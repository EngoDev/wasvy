from typing import TypeVar, Generic, Union, Optional, Protocol, Tuple, List, Any, Self
from types import TracebackType
from enum import Flag, Enum, auto
from dataclasses import dataclass
from abc import abstractmethod
import weakref

from ..types import Result, Ok, Err, Some


@dataclass
class Component:
    """
    This is the translation object between bevy Rust `Component` and a bevy `Component` that is registerd in WASM.
    
    `value` is the JSON serialized version of the actual component that is being passed between WASM and Bevy.
    So for every instance of `component` make sure you deserialize it yourself to the struct that it actually is.
    """
    id: int
    value: str

@dataclass
class Query:
    """
    This is the translation object between bevy `Query` and WASM query that can be used for registering systems.
    
    For example if we had the following bevy `Query`: `Query<&Name, Without<Transform>`
    It would look like this as a WASM `query` object:
    `query {
    components: [functions:get-component-id("Name")],
    without: [functions:get-component-id("Transform")],
    }
    """
    components: List[int]
    with_: List[int]
    without: List[int]

@dataclass
class QueryResultEntry:
    """
    This is one row for a query parameter
    
    For example if we take the following bevy system:
    
    fn system(first_query: Query<(&Name, &Transform)>) {}
    
    query-result-entry is equal to one entry in `first_query`
    """
    components: List[Component]
    entity: int


