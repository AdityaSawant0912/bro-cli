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
    if(insert(db=DEFAULT_DB, table=CMD, alias=alias, cmd=full_cmd)):
        typer.echo(f"Added new command `{full_cmd}` with alias `{alias}` into database.")
        typer.echo(f"Use `bro run {alias}` to execute.")
    else:
        typer.echo(f"Failed to add new command into database.")
