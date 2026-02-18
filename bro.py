#!/usr/bin/env python
import subprocess
import sys
import os
import typer
from typing import Optional
from db import Database
from constants import DEFAULT_DB, TABLE_CMD, TABLES, TABLE_INFO, TABLE_ALIAS
from config import (
    load_project_config,
    init_project_config,
    add_to_project_config,
    delete_from_project_config,
    update_project_config,
    get_project_config_path,
    find_project_config,
)

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
    
    # Project-local flag
    local: bool = typer.Option(False, "--local", help="Use project-local .bro file instead of global database"),
    init: bool = typer.Option(False, "--init", help="Initialize .bro config file in current directory"),
    force: bool = typer.Option(False, "--force", help="Force overwrite (use with --init)"),
    
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
        bro --init                      # Initialize .bro config file
        bro -a deploy "npm run deploy"  # Add command (global)
        bro -a run "python main.py" --local  # Add to project .bro
        bro -a script -py ./script.py   # Add Python script
        bro -c script,deploy            # Chain alias
        bro -u deploy "yarn deploy"     # Update command
        bro -d deploy                   # Delete alias
        bro -d run --local              # Delete from project .bro
        bro -l                          # List all (global + project)
        bro -i backup                   # Show info
        bro -s docker                   # Search aliases
    """
    if ctx.invoked_subcommand != None:
        return
    
    # Initialize .bro file
    if init:
        _init_config(force)
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
        _add_alias(alias, cmd=cmd, py=py, js=js, local=local)
        return
    
    # Delete alias
    if delete:
        _delete_alias(alias, local=local)
        return
    
    # Update alias
    if update:
        if not cmd and not py and not js:
            typer.echo("Error: Please provide a command or script path to update", err=True)
            raise typer.Exit(code=1)
        _update_alias(alias, cmd=cmd, py=py, js=js, local=local)
        return
    
    # Show info
    if info:
        _show_info(alias)
        return
    
    # Default: Execute the alias
    _execute_alias(alias, args)

def _add_alias(alias: str, local: bool = False, **script_flags):
    """Add a new alias"""
    
    # Handle local (project) config
    if local:
        # Only support simple commands for local config
        cmd = script_flags.get('cmd')
        if not cmd:
            typer.echo("Error: --local flag only supports simple commands (use -a <alias> <command> --local)", err=True)
            raise typer.Exit(code=1)
        
        success = add_to_project_config(alias, cmd)
        if success:
            config_path = get_project_config_path()
            typer.echo(f"✓ Added local alias '{alias}' → {cmd}")
            typer.echo(f"  (in {config_path})")
        else:
            typer.echo(f"Failed to add local alias (check if .bro file exists or run 'bro init')", err=True)
            raise typer.Exit(code=1)
        return
    
    # Handle global database (existing logic)
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
        typer.echo(f"✓ Added {table_config['type'].lower()} '{alias}' → {value}")
    else:
        typer.echo(f"Failed to add alias '{alias}' (may already exist)", err=True)
        raise typer.Exit(code=1)


def _delete_alias(alias: str, local: bool = False):
    """Delete an alias"""
    
    # Handle local (project) config
    if local:
        success = delete_from_project_config(alias)
        if success:
            typer.echo(f"✓ Deleted local alias '{alias}'")
        else:
            typer.echo(f"Local alias '{alias}' not found in .bro file", err=True)
            raise typer.Exit(code=1)
        return
    
    # Handle global database
    deleted = False
    for table in TABLES:
        deleted = db.delete(table, f"alias = '{alias}'")
        if deleted: break

    if deleted:
        typer.echo(f"✓ Deleted global alias '{alias}'")
    else:
        typer.echo(f"Global alias '{alias}' not found", err=True)
        raise typer.Exit(code=1)


def _update_alias(alias: str, local: bool = False, **kwargs):
    """Update an existing alias"""
    
    # Handle local (project) config
    if local:
        cmd = kwargs.get('cmd')
        if not cmd:
            typer.echo("Error: --local flag only supports simple commands", err=True)
            raise typer.Exit(code=1)
        
        success = update_project_config(alias, cmd)
        if success:
            typer.echo(f"✓ Updated local alias '{alias}' → {cmd}")
        else:
            typer.echo(f"Local alias '{alias}' not found in .bro file", err=True)
            raise typer.Exit(code=1)
        return
    
    # Handle global database (existing logic)
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
        typer.echo(f"✓ Updated {info['type'].lower()} '{alias}' → {value}")
    else:
        typer.echo(f"Alias '{alias}' not found", err=True)
        raise typer.Exit(code=1)


def _list_aliases():
    """List all aliases (global + project)"""
    
    # Show project config first
    project_config = load_project_config()
    config_path = find_project_config()
    
    if project_config:
        typer.echo(f"\n📁 Project Aliases (from {config_path}):")
        for alias, command in sorted(project_config.items()):
            typer.echo(f"  {alias:<20} → {command}")
        typer.echo()
    
    # Show global aliases
    all_empty = True
    
    for table in TABLES:
        if table not in TABLE_INFO:
            continue
            
        results = db.find(table, "1=1")  # Get all
        
        if results:
            all_empty = False
            info = TABLE_INFO[table]
            typer.echo(f"🌍 Global {info['label']}:")
            for row in results:
                value = row[info['value_key']]
                # Mark if shadowed by project config
                marker = " (shadowed)" if project_config and row['alias'] in project_config else ""
                typer.echo(f"  {row['alias']:<20} → {value}{marker}")
    
    if all_empty and not project_config:
        typer.echo("No aliases found. Add one with: bro -a <alias> <command>")
        typer.echo("Or create a project config with: bro init")
        return
    
    typer.echo()


def _search_aliases(keyword: str):
    """Search aliases by keyword (global + project)"""

    found_any = False
    
    # Search project config
    project_config = load_project_config()
    if project_config:
        matches = {k: v for k, v in project_config.items() 
                   if keyword.lower() in k.lower() or keyword.lower() in v.lower()}
        
        if matches:
            found_any = True
            typer.echo(f"\n🔍 Results for '{keyword}':\n")
            typer.echo("📁 Project Aliases:")
            for alias, command in sorted(matches.items()):
                typer.echo(f"  {alias:<20} → {command}")
            typer.echo()
    
    # Search global database
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
            
            typer.echo(f"🌍 Global {info['label']}:")
            for row in results:
                value = row[info['value_key']]
                typer.echo(f"  {row['alias']:<20} → {value}")
            typer.echo()
    
    if not found_any:
        typer.echo(f"No aliases matching '{keyword}'")


def _show_info(alias: str):
    """Show detailed info about an alias (checks project first, then global)"""
    
    # Check project config first
    project_config = load_project_config()
    if project_config and alias in project_config:
        config_path = find_project_config()
        typer.echo(f"\n📁 Project Alias: {alias}")
        typer.echo(f"   Source: {config_path}")
        typer.echo(f"   Command: {project_config[alias]}\n")
        
        # Also check if there's a global alias with same name
        for table in TABLES:
            if table not in TABLE_INFO:
                continue
            result = db.find_one(table, f"alias = '{alias}'")
            if result:
                typer.echo(f"   ⚠️  Note: Global alias '{alias}' is being shadowed by this project alias")
                break
        return
    
    # Check global database
    for table in TABLES:
        if table not in TABLE_INFO:
            continue
        
        result = db.find_one(table, f"alias = '{alias}'")
        
        if result:
            info = TABLE_INFO[table]
            typer.echo(f"\n🌍 Global {info['label'][:-1]}: {alias}")
            typer.echo(f"   Type: {info['label'][:-1]}")
            typer.echo(f"   {info['value_key'].capitalize()}: {result[info['value_key']]}\n")
            return
    
    # Not found anywhere
    typer.echo(f"Alias '{alias}' not found", err=True)
    raise typer.Exit(code=1)


def _chain_aliases(aliases: str, extra_args: Optional[str]):
    """Execute sequence of aliases (commands or scripts)"""
    _aliases = aliases.split(",")
    commands: list[tuple[str, str]] = []
    
    # Load project config once
    project_config = load_project_config()
    
    for alias in _aliases:
        # Check project config first
        if project_config and alias in project_config:
            command = project_config[alias]
            commands.append((command, f"project alias '{alias}'"))
            continue
        
        # Check global database
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
                break

        if not got_result:
            typer.echo(f"Alias '{alias}' not found", err=True)
            typer.echo("Tip: Use 'bro -l' to list all aliases")
            raise typer.Exit(code=1)
    
    for cmd in commands:
        execute_command(cmd[0], cmd[1])


def _execute_alias(alias: str, extra_args: Optional[str]):
    """Execute an alias (checks project first, then global)"""
    
    # Check project config first
    project_config = load_project_config()
    if project_config and alias in project_config:
        command = project_config[alias]
        
        # Append extra args if provided
        if extra_args:
            command = f"{command} {extra_args.strip()}"
        
        execute_command(command, f"project alias '{alias}'")
        return
    
    # Check global database
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


def _init_config(force: bool = False):
    """Initialize a .bro config file in the current directory"""
    
    config_path = get_project_config_path()
    
    if config_path.exists() and not force:
        typer.echo(f"✓ .bro file already exists at {config_path}")
        typer.echo("  Use 'bro --init --force' to overwrite")
        return
    
    success = init_project_config(force=force)
    
    if success:
        typer.echo(f"✓ Created .bro file at {config_path}")
        typer.echo("\nExample usage:")
        typer.echo("  bro -a run 'python main.py' --local    # Add local alias")
        typer.echo("  bro run                                 # Execute local alias")
        typer.echo(f"\nEdit {config_path} to customize your project aliases")
    else:
        typer.echo("Failed to create .bro file", err=True)
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