import os
from constants import *
import typer
from db import delete

app = typer.Typer()

@app.command('cmd', help='Removes existing custom command.')
def delete_cmd(alias: str):
    if(delete(db=DEFAULT_DB, table=CMD, where=f"alias='{alias}'")):
      typer.echo(f"Removing existing command with alias `{alias}` from the database")
    else:
      typer.echo(f"Failed to remove existing command from the database.")
