import os
from constants import *
import typer
from db import insert

app = typer.Typer()

@app.command('cmd', help='Add new custom command.')
def add_cmd(alias: str, cmd:str):
    if(insert(db=DEFAULT_DB, table=CMD, alias=alias, cmd=cmd)):
      typer.echo(f"Added new command `{cmd}` with alias `{alias}` into database.")
      typer.echo(f"Use `bro run {alias}` to execute.")
    else:
      typer.echo(f"Failed to add new command into database.")
