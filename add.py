import os
from constants import *
import typer
from db import insert
from typing import List  # Import List for compatibility
import shlex  # Import shlex for escaping arguments

app = typer.Typer()

@app.command('cmd', help='Add new custom command.')
def add_cmd(alias: str, cmd: List[str] = typer.Argument(..., help="Command to execute, enclose in quotes if it contains spaces")):
    # Escape each part of the command to handle special characters
    full_cmd = " ".join(cmd).strip() # Combine all parts of the command into a single string
    if(insert(db=DEFAULT_DB, table=TABLE_CMD, alias=alias, cmd=full_cmd)):
        typer.echo(f"Added new command `{full_cmd}` with alias `{alias}` into database.")
        typer.echo(f"Use `bro run {alias}` to execute.")
    else:
        typer.echo(f"Failed to add new command into database.")

@app.command('py', help='Add new custom python script.')
def add_py(alias: str, path: List[str] = typer.Argument(..., help="Path to python script")):
    # Escape each part of the path to handle special characters
    full_path = " ".join(path).strip() # Combine all parts of the path into a single string
    if(insert(db=DEFAULT_DB, table=TABLE_PYTHON, alias=alias, path=full_path)):
        typer.echo(f"Added new python script at `{full_path}` with alias `{alias}` into database.")
        typer.echo(f"Use `bro py {alias}` to execute.")
    else:
        typer.echo(f"Failed to add new python script into database.")
