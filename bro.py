#!/usr/bin/env python
from constants import *
import subprocess
import typer
import add
import remove
from db import *

TYPER_OPTIONS = {
    "help": "help",
    "hide": "--hide",
}

app = typer.Typer(help="CLI")

@app.command('hello', help='Bro says Hello.')
def hello():
    typer.echo("Hello Bro!")
    typer.echo("Type 'bro --help' to begin")


@app.command('run', help='Bro executes your custom commands.')
def bro(alias: str):
    
    checkDB(DEFAULT_DB)
    result = find(DEFAULT_DB, CMD, f"alias = '{alias}'")

    if not result:
        typer.echo(f"Command '{alias}' not found.")
        raise typer.Exit()

    _, command = result[0]

    try:
        subprocess.run(command, shell=True, check=True)
    except subprocess.CalledProcessError as e:
        typer.echo(f"Error executing command: {e}")

app.add_typer(add.app, name="add", help="Add stuff")
app.add_typer(remove.app, name="delete", help="Delete stuff")

if __name__ == "__main__":
    app()