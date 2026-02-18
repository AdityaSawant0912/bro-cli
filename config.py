#!/usr/bin/env python
"""
Project-local configuration handler for .bro files
"""
import os
from typing import Optional, Dict
from pathlib import Path

try:
    import tomli
    import tomli_w
    TOML_AVAILABLE = True
except ImportError:
    TOML_AVAILABLE = False
    print("Warning: tomli/tomli_w not installed. Install with: pip install tomli tomli-w")


CONFIG_FILENAME = ".bro"


def find_project_config() -> Optional[Path]:
    """
    Walk up directory tree to find .bro file
    Returns the path to .bro if found, None otherwise
    """
    current = Path.cwd()
    
    # Walk up to root
    while current != current.parent:
        config_path = current / CONFIG_FILENAME
        if config_path.exists() and config_path.is_file():
            return config_path
        current = current.parent
    
    return None


def load_project_config() -> Optional[Dict[str, str]]:
    """
    Load project-local configuration from .bro file
    Returns dict of aliases or None if no config found
    """
    if not TOML_AVAILABLE:
        return None
    
    config_path = find_project_config()
    if not config_path:
        return None
    
    try:
        with open(config_path, 'rb') as f:
            data = tomli.load(f)
            # Support both root-level and [aliases] section
            if 'aliases' in data:
                return data['aliases']
            return data
    except Exception as e:
        print(f"Error loading .bro config: {e}")
        return None


def get_project_config_path() -> Path:
    """Get the .bro path in current directory"""
    return Path.cwd() / CONFIG_FILENAME


def init_project_config(force: bool = False) -> bool:
    """
    Initialize a .bro file in current directory
    Returns True if created, False if already exists (and not forced)
    """
    if not TOML_AVAILABLE:
        return False
    
    config_path = get_project_config_path()
    
    if config_path.exists() and not force:
        return False
    
    # Create template config
    template = {
        "aliases": {
            "run": "echo 'Add your run command here'",
            "test": "echo 'Add your test command here'",
            "build": "echo 'Add your build command here'",
        }
    }
    
    try:
        with open(config_path, 'wb') as f:
            tomli_w.dump(template, f)
        return True
    except Exception as e:
        print(f"Error creating .bro config: {e}")
        return False


def add_to_project_config(alias: str, command: str) -> bool:
    """
    Add or update an alias in the local .bro file
    Creates the file if it doesn't exist
    Returns True on success
    """
    if not TOML_AVAILABLE:
        return False
    
    config_path = get_project_config_path()
    
    # Load existing or create new
    if config_path.exists():
        try:
            with open(config_path, 'rb') as f:
                data = tomli.load(f)
        except Exception as e:
            print(f"Error reading .bro config: {e}")
            return False
    else:
        data = {"aliases": {}}
    
    # Ensure aliases section exists
    if 'aliases' not in data:
        data['aliases'] = {}
    
    # Add/update alias
    data['aliases'][alias] = command
    
    # Write back
    try:
        with open(config_path, 'wb') as f:
            tomli_w.dump(data, f)
        return True
    except Exception as e:
        print(f"Error writing .bro config: {e}")
        return False


def delete_from_project_config(alias: str) -> bool:
    """
    Delete an alias from local .bro file
    Returns True if deleted, False if not found or error
    """
    if not TOML_AVAILABLE:
        return False
    
    config_path = get_project_config_path()
    
    if not config_path.exists():
        return False
    
    try:
        with open(config_path, 'rb') as f:
            data = tomli.load(f)
        
        # Check if alias exists
        if 'aliases' not in data or alias not in data['aliases']:
            return False
        
        # Remove alias
        del data['aliases'][alias]
        
        # Write back
        with open(config_path, 'wb') as f:
            tomli_w.dump(data, f)
        
        return True
    except Exception as e:
        print(f"Error updating .bro config: {e}")
        return False


def update_project_config(alias: str, command: str) -> bool:
    """
    Update an existing alias in local .bro file
    Returns True if updated, False if alias doesn't exist
    """
    if not TOML_AVAILABLE:
        return False
    
    config_path = get_project_config_path()
    
    if not config_path.exists():
        return False
    
    try:
        with open(config_path, 'rb') as f:
            data = tomli.load(f)
        
        # Check if alias exists
        if 'aliases' not in data or alias not in data['aliases']:
            return False
        
        # Update alias
        data['aliases'][alias] = command
        
        # Write back
        with open(config_path, 'wb') as f:
            tomli_w.dump(data, f)
        
        return True
    except Exception as e:
        print(f"Error updating .bro config: {e}")
        return False


def get_alias_source(alias: str) -> Optional[str]:
    """
    Determine if an alias comes from project or global config
    Returns 'project', 'global', or None
    """
    config = load_project_config()
    if config and alias in config:
        return 'project'
    return None  # Caller should check global DB