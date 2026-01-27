#!/usr/bin/env python
import subprocess
import sys
import os
import typer
from typing import Optional
from db import Database
from constants import DEFAULT_DB, TABLE_CMD, TABLES, TABLE_INFO, TABLE_ALIAS

app = typer.Typer(add_completion=False, no_args_is_help=True)
db = Database(DEFAULT_DB)


def execute_command(command: str, description: str = "command"):
    """Execute a shell command with error handling"""
    typer.echo(f"Executing {description}: {command}")
    try:
        subprocess.run(command, shell=True, check=True)
    except subprocess.CalledProcessError as e:
        typer.echo(f"Error executing {description}: {e}", err=True)
        raise typer.Exit(code=1)


@app.callback(invoke_without_command=True)
def main(
    ctx: typer.Context,
    alias: Optional[str] = typer.Argument(None, help="Alias name"),
    cmd: Optional[str] = typer.Argument(None, help="Command to execute"),
    
    # CRUD flags (boolean)
    chain: bool = typer.Option(False, "-c", "--chain", help="Chain alias to execute"),
    add: bool = typer.Option(False, "-a", "--add", help="Add new alias"),
    delete: bool = typer.Option(False, "-d", "--delete", help="Delete an alias"),
    update: bool = typer.Option(False, "-u", "--update", help="Update an alias"),
    list_all: bool = typer.Option(False, "-l", "--list", help="List all aliases"),
    info: bool = typer.Option(False, "-i", "--info", help="Show info about an alias"),
    search: Optional[str] = typer.Option(None, "-s", "--search", help="Search aliases by keyword"),
    
    # Script type flags
    py: Optional[str] = typer.Option(None, "-py", "--python", help="Python script path"),
    js: Optional[str] = typer.Option(None, "-js", "--javascript", help="JavaScript script path"),
    
    # Additional args for execution
    args: Optional[str] = typer.Option(None, help="Additional arguments for command execution"),
):
    """
    bro - Your personal CLI assistant
    
    Examples:
        bro backup                      # Execute 'backup' alias
        bro -a deploy "npm run deploy"  # Add command
        bro -a script -py ./script.py   # Add Python script
        bro -c script,deploy            # Chain alias
        bro -u deploy "yarn deploy"     # Update command
        bro -d deploy                   # Delete alias
        bro -l                          # List all
        bro -i backup                   # Show info
        bro -s docker                   # Search aliases
    """
    if ctx.invoked_subcommand != None:
        return
    
    # List all aliases
    if list_all:
        _list_aliases()
        return
    
    # Search aliases
    if search:
        _search_aliases(search)
        return
    
    
    # Require alias for most operations
    if alias is None and not list_all and not search:
        typer.echo("Error: Please provide an alias or use -l to list all.", err=True)
        raise typer.Exit(code=1)
    
    # Chain aliases
    if chain:
        _chain_aliases(alias, args)
        return
    
    # Add new alias
    if add:
        if not cmd and not py and not js:
            typer.echo("Error: Please provide a command or script path", err=True)
            raise typer.Exit(code=1)
        _add_alias(alias, cmd=cmd, py=py, js=js)
        return
    
    # Delete alias
    if delete:
        _delete_alias(alias)
        return
    
    # Update alias
    if update:
        if not cmd and not py and not js:
            typer.echo("Error: Please provide a command or script path to update", err=True)
            raise typer.Exit(code=1)
        _update_alias(alias, cmd=cmd, py=py, js=js)
        return
    
    # Show info
    if info:
        _show_info(alias)
        return
    
    # Default: Execute the alias
    _execute_alias(alias, args)

def _add_alias(alias: str, **script_flags):
    """Add a new alias"""
    table = None
    value = None
    
    for flag, flag_value in script_flags.items():
        if flag_value is None:
            continue

        # Check if flag matches any table alias
        if flag not in TABLE_ALIAS:
            continue
        
        table = TABLE_ALIAS[flag]
        table_config = TABLE_INFO[table]
        
        # Validate extension if validator exists
        if table_config.get('validator') and not table_config['validator'](flag_value):
            ext = table_config.get('extension', '')
            typer.echo(f"Error: {table_config['type']} must have {ext} extension", err=True)
            raise typer.Exit(code=1)
        
        # Validate file exists (for scripts)
        if flag != TABLE_CMD and not os.path.isfile(flag_value):
            typer.echo(f"Error: File not found: {flag_value}", err=True)
            raise typer.Exit(code=1)
        
        value = flag_value
        break
    
    if not table or not value:
        typer.echo("Error: No valid command or script provided", err=True)
        raise typer.Exit(code=1)
    
    # Insert the alias
    table_config = TABLE_INFO[table]
    success = db.insert(table, alias=alias, **{table_config['value_key']: value})
    
    if success:
        typer.echo(f"Added {table_config['type'].lower()} '{alias}' → {value}")
    else:
        typer.echo(f"Failed to add alias '{alias}' (may already exist)", err=True)
        raise typer.Exit(code=1)


def _delete_alias(alias: str):
    """Delete an alias"""
    # Try all tables
    deleted = False
    for table in TABLES:
        deleted = db.delete(table, f"alias = '{alias}'")
        if deleted: break

    if deleted:
        typer.echo(f"Deleted alias '{alias}'")
    else:
        typer.echo(f"Alias '{alias}' not found", err=True)
        raise typer.Exit(code=1)


def _update_alias(alias: str, **kwargs):
    """Update an existing alias"""
    
    # Find which flag was provided
    table = None
    value = None
    
    for flag, flag_value in kwargs.items():
        if flag_value:
            table = TABLE_ALIAS.get(flag)
            value = flag_value
            break
    
    if table is None or value is None:
        typer.echo("Error: No valid update value provided", err=True)
        raise typer.Exit(code=1)
    
    # Get table info
    if table not in TABLE_INFO:
        typer.echo(f"Unsupported table type", err=True)
        raise typer.Exit(code=1)
    
    info = TABLE_INFO[table]
    
    # Update the alias
    success = db.update(table, f"alias = '{alias}'", **{info['value_key']: value})
    
    if success:
        typer.echo(f"Updated {info['type'].lower()} '{alias}' → {value}")
    else:
        typer.echo(f"Alias '{alias}' not found", err=True)
        raise typer.Exit(code=1)


def _list_aliases():
    """List all aliases"""
    
    all_empty = True
    
    for table in TABLES:
        if table not in TABLE_INFO:
            continue
            
        results = db.find(table, "1=1")  # Get all
        
        if results:
            all_empty = False
            info = TABLE_INFO[table]
            typer.echo(f"\n{info['label']}:")
            for row in results:
                value = row[info['value_key']]
                typer.echo(f"  {row['alias']:<20} → {value}")
    
    if all_empty:
        typer.echo("No aliases found. Add one with: bro -a <alias> <command>")
        return
    
    typer.echo()


def _search_aliases(keyword: str):
    """Search aliases by keyword"""

    found_any = False
    
    for table in TABLES:
        if table not in TABLE_INFO:
            continue
        
        info = TABLE_INFO[table]
        # Search in both alias and value columns
        where_clause = f"alias LIKE '%{keyword}%' OR {info['value_key']} LIKE '%{keyword}%'"
        results = db.find(table, where_clause)
        
        if results:
            if not found_any:
                typer.echo(f"\n🔍 Results for '{keyword}':\n")
                found_any = True
            
            typer.echo(f"{info['label']}:")
            for row in results:
                value = row[info['value_key']]
                typer.echo(f"  {row['alias']:<20} → {value}")
            typer.echo()
    
    if not found_any:
        typer.echo(f"No aliases matching '{keyword}'")


def _show_info(alias: str):
    """Show detailed info about an alias"""
    for table in TABLES:
        if table not in TABLE_INFO:
            continue
        
        result = db.find_one(table, f"alias = '{alias}'")
        
        if result:
            info = TABLE_INFO[table]
            typer.echo(f"\n {info['label'][:-1]}: {alias}")  # Remove 's' from label
            typer.echo(f"   Type: {info['label'][:-1]}")
            typer.echo(f"   {info['value_key'].capitalize()}: {result[info['value_key']]}\n")
            return
    
    # Not found in any table
    typer.echo(f"Alias '{alias}' not found", err=True)
    raise typer.Exit(code=1)


def _chain_aliases(aliases: str, extra_args: Optional[str]):
    """Execute sequence of aliases (commands or scripts)"""
    _aliases = aliases.split(",")
    commands: list[tuple[str, str]] = []
    for alias in _aliases:
        got_result = False
        for table in TABLES:
            if table not in TABLE_INFO:
                continue
            
            result = db.find_one(table, f"alias = '{alias}'")
            if result:
                info = TABLE_INFO[table]
                value = result[info['value_key']]
                
                # Build command using executor
                command = info['executor'](value)

                commands.append((command, f"{info['type'].lower()} '{alias}'"))
                got_result = True

        if not got_result:
            typer.echo(f"Alias '{alias}' not found", err=True)
            typer.echo("Tip: Use 'bro -l' to list all aliases")
            raise typer.Exit(code=1)
    
    for cmd in commands:
        execute_command(cmd[0], cmd[1])
    print(aliases)


def _execute_alias(alias: str, extra_args: Optional[str]):
    """Execute an alias (command or script)"""
    for table in TABLES:
        if table not in TABLE_INFO:
            continue
        
        result = db.find_one(table, f"alias = '{alias}'")
        
        if result:
            info = TABLE_INFO[table]
            value = result[info['value_key']]
            
            # Build command using executor
            command = info['executor'](value)
            
            # Append extra args if provided
            if extra_args:
                command = f"{command} {extra_args.strip()}"
            
            # Execute
            execute_command(command, f"{info['type'].lower()} '{alias}'")
            return
    
    # Not found in any table
    typer.echo(f"Alias '{alias}' not found", err=True)
    typer.echo("Tip: Use 'bro -l' to list all aliases")
    raise typer.Exit(code=1)


@app.command()
def hello():
    """Bro says hello"""
    typer.echo("Hello Bro!")
    typer.echo("Type 'bro --help' to begin")


# @app.command()
# def bye():
#     """Shutdown the computer"""
#     if not typer.confirm("Are you really leaving me?"):
#         typer.echo("Good. Stay.")
#         raise typer.Exit()
    
#     typer.echo("Aww, Okay bye...")
#     try:
#         subprocess.run("shutdown /s /t 0", shell=True, check=True)
#     except Exception as e:
#         typer.echo(f"Shutdown failed: {e}", err=True)


if __name__ == "__main__":
    try:
        app()
    finally:
        # Clean up database connection on exit
        db.close()