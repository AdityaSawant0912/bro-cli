#!/usr/bin/env python

import os
import typer


TYPER_OPTIONS = {
    "help": "help",
    "hide": "--hide",
}


app = typer.Typer(help="CLI")

@app.command('hello', help='Bro says Hello.')
def hello():
    typer.echo("Hello Bro!")
    typer.echo("Type 'bro --help' to begin")
    


if __name__ == "__main__":
    app()