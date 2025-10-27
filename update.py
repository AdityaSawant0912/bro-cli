import os
from constants import *
import typer
from db import update

app = typer.Typer()

@app.command('cmd', help='Update existing custom command.')
def update_cmd(alias: str, updated_cmd:str):
    if(update(db=DEFAULT_DB, table=TABLE_CMD, where=f"alias = {alias}", cmd=updated_cmd)):
      typer.echo(f"Updated command with alias `{alias}` into database.")
      typer.echo(f"Use `bro run {alias}` to execute.")
    else:
      typer.echo(f"Failed to update new command into database.")

@app.command('py', help='Update existing custom command.')
def update_py(alias: str, updated_path:str):
    if(update(db=DEFAULT_DB, table=TABLE_PYTHON, where=f"alias = {alias}", cmd=updated_path)):
      typer.echo(f"Updated path with alias `{alias}` into database.")
      typer.echo(f"Use `bro py {alias}` to execute.")
    else:
      typer.echo(f"Failed to update new path into database.")
