import os
from constants import *
import typer
from db import update

app = typer.Typer()

@app.command('cmd', help='Update existing custom command.')
def update_cmd(alias: str, updated_cmd:str):
    if(update(db=DEFAULT_DB, table=CMD, where=f"alias = {alias}", cmd=cmd)):
      typer.echo(f"Updated command with alias `{alias}` into database.")
      typer.echo(f"Use `bro run {alias}` to execute.")
    else:
      typer.echo(f"Failed to update new command into database.")
