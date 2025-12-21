#!/usr/bin/env python
from constants import *
import subprocess
import typer
import add
import remove
from db import *
from typing import List  # Import List for compatibility
import shlex  # Import shlex for escaping arguments

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
def run(alias: str, args: str = typer.Argument(default='', help="Additional arguments to append to the stored command")):
    checkDB(DEFAULT_DB)
    result = find(DEFAULT_DB, TABLE_CMD, f"alias = '{alias}'")

    if not result:
        typer.echo(f"Command '{alias}' not found.")
        raise typer.Exit()

    _, command = result[0]

    # Escape additional arguments to handle special characters
    escaped_args = args.strip()
    full_command = f"{command} {escaped_args}"

    # Log the command being executed
    typer.echo(f"Executing command: {full_command}")
    
    try:
        subprocess.run(full_command, shell=True, check=True)
    except subprocess.CalledProcessError as e:
        typer.echo(f"Error executing command: {e}")

@app.command('py', help='Bro executes your custom commands.')
def bro(alias: str, args: str = typer.Argument(default='', help="Additional arguments to append to the stored command")):
    checkDB(DEFAULT_DB)
    result = find(DEFAULT_DB, TABLE_PYTHON, f"alias = '{alias}'")

    if not result:
        typer.echo(f"Command '{alias}' not found.")
        raise typer.Exit()

    _, path = result[0]

    # Escape additional arguments to handle special characters
    escaped_args = args.strip()
    full_command = f"python {path} {escaped_args}"

    # Log the command being executed
    typer.echo(f"Executing script: {full_command}")
    
    try:
        subprocess.run(full_command, shell=True, check=True)
    except subprocess.CalledProcessError as e:
        typer.echo(f"Error executing command: {e}")
app.add_typer(add.app, name="add", help="Add stuff")
app.add_typer(remove.app, name="delete", help="Delete stuff")

@app.command('bye', help="Bro says bye and shuts down the computer.")
def bye():
    if not typer.confirm("Are you really leaving me?"):
        typer.echo("Good. Stay.")
        raise typer.Exit()

    typer.echo("Aww, Okay bye...")

    cmd = "shutdown /s /t 0"
    try:
        subprocess.run(cmd, shell=True, check=True)
    except subprocess.CalledProcessError as e:
        typer.echo(f"Error executing shutdown command: {e}")
    except Exception as e:
        typer.echo(f"Shutdown failed: {e}")

if __name__ == "__main__":
    app()