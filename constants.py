from typing import Callable, TypedDict
from collections import defaultdict


class TableConfig(TypedDict):
    alias: str
    extension: str
    label: str
    type: str
    value_key: str
    value_label: str
    validator: Callable[[str], bool]
    executor: Callable[[str], str]


default_dbs = {
    "main": "main_db",
}

default_schemas = {
    "main": "schema.sql",
}

# DB
PATH_TO_DB = 'R:\\bro-cli\\'
DEFAULT_DB = 'main'


# Tables
TABLES: list[str] = []
TABLE_INFO: dict[str, TableConfig] = defaultdict()
TABLE_ALIAS: dict[str, str] = defaultdict()


TABLE_CMD = 'cmd'
TABLES.append(TABLE_CMD)
TABLE_INFO[TABLE_CMD] = {
    "alias": "cmd",
    "extension": "",
    "label": "Commands",
    "type": "Shell Command",
    "value_key": "cmd",
    "value_label": "Runs",
    "validator": lambda value: True,
    "executor": lambda value: value
}

TABLE_PYTHON = 'python'
TABLES.append(TABLE_PYTHON)
TABLE_INFO[TABLE_PYTHON] = {
    "alias": "py",
    "extension": ".py",
    "label": "Python Scripts",
    "type": "Python Script",
    "value_key": "path",
    "value_label": "Path",
    "validator": lambda value: True,
    "executor": lambda value: f"python {value}"
}

TABLE_JS = 'javascript'
TABLES.append(TABLE_JS)
TABLE_INFO[TABLE_JS] = {
    "alias": "js",
    "extension": ".js",
    "label": "JavaScript Scripts",
    "type": "JavaScript Script",
    "value_key": "path",
    "value_label": "Path",
    "validator": lambda value: True,
    "executor": lambda value: f"node {value}"
}

TABLE_PS = 'powershell'
TABLES.append(TABLE_PS)
TABLE_INFO[TABLE_PS] = {
    "alias": "ps",
    "extension": ".ps",
    "label": "Powershell Scripts",
    "type": "Powershell Script",
    "value_key": "path",
    "value_label": "Path",
    "validator": lambda value: True,
    "executor": lambda value: f"powershell {value}"
}


for table in TABLES:
    TABLE_ALIAS[TABLE_INFO[table]["alias"]] = table