import json

from typing import List
from dataclasses import dataclass, asdict

import guest
from guest.imports import functions as host_functions


@dataclass
class PythonComponent:
    kind: str


class Guest(guest.Guest):
    def hello_world(self) -> str:
        return "Hello World From Python!"

    def setup(self):
        id1 = host_functions.register_component(PythonComponent.__name__)
        host_functions.register_system(
            "print-first-component-system",
            [guest.types.Query([PythonComponent.__name__], [], [])],
        )

        serialized_component = json.dumps(asdict(PythonComponent(kind="Boa")))

        host_functions.spawn(
            [guest.types.Component(PythonComponent.__name__, serialized_component)]
        )

    def print_first_component_system(
        self, params: List[List[guest.types.QueryResultEntry]]
    ):
        python_component_query = params[0]

        for row in python_component_query:
            print(f"Python Entity: {row.entity}")
            python_component_dict = json.loads(row.components[0].value)
            python_component = PythonComponent(**python_component_dict)
            print(f"Python component: {python_component}")
