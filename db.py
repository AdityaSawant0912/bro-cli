import sqlite3
import os
from typing import Optional, List, Tuple, Any
from constants import PATH_TO_DB, TABLES, default_dbs, default_schemas

class Database:
    _instances: dict[str, 'Database'] = {}  # Store instances per database
    
    def __new__(cls, db_key: str):
        """Singleton pattern - one instance per database"""
        if db_key not in cls._instances:
            instance = super().__new__(cls)
            cls._instances[db_key] = instance
        return cls._instances[db_key]
    
    def __init__(self, db_key: str):
        """Initialize only once per database"""
        if hasattr(self, '_initialized'):
            return
            
        if db_key not in default_dbs:
            raise ValueError(f"Invalid database key: {db_key}")
        
        self.db_key = db_key
        self.db_name = default_dbs[db_key]
        self.db_path = os.path.join(PATH_TO_DB, self.db_name + ".db")
        self._connection: Optional[sqlite3.Connection] = None
        self._initialized = True
        
        # Initialize database if needed
        self._check_and_init()
    
    @property
    def connection(self) -> sqlite3.Connection:
        """Lazy connection - only connect when needed"""
        if self._connection is None:
            self._connection = sqlite3.connect(self.db_path)
            self._connection.row_factory = sqlite3.Row  # Access columns by name
        return self._connection
    
    @property
    def cursor(self) -> sqlite3.Cursor:
        """Get cursor from active connection"""
        return self.connection.cursor()
    
    def _check_and_init(self) -> None:
        """Check if DB exists and initialize if needed"""
        if not os.path.isfile(self.db_path):
            print(f"Database '{self.db_name}' not found. Initializing...")
            self._init_schema()
            return
        
        # Check if tables exist
        cur = self.cursor
        cur.execute("SELECT name FROM sqlite_master WHERE type='table';")
        tables = cur.fetchall()
        
        if len(tables) != len(TABLES):
            self._init_schema()
    
    def _init_schema(self) -> None:
        """Initialize database schema"""
        schema_file = default_schemas[self.db_key]
        path_to_schema = os.path.join(PATH_TO_DB, schema_file)
        
        if not os.path.isfile(path_to_schema):
            raise FileNotFoundError(f"Schema file not found: {path_to_schema}")
        
        with open(path_to_schema, 'r') as f:
            schema_sql = f.read()
        
        self.cursor.executescript(schema_sql)
        self.connection.commit()
        print(f"Database '{self.db_name}' initialized successfully.")
    
    def find(self, table: str, where: str, select: str = '*') -> List[sqlite3.Row]:
        """Query records"""
        query = f"SELECT {select} FROM {table} WHERE {where};"
        try:
            cur = self.cursor
            cur.execute(query)
            return cur.fetchall()
        except sqlite3.Error as e:
            print(f"Error fetching data: {e}")
            return []
    
    def find_one(self, table: str, where: str, select: str = '*') -> Optional[sqlite3.Row]:
        """Query single record"""
        results = self.find(table, where, select)
        return results[0] if results else None
    
    def insert(self, table: str, **kw: Any) -> bool:
        """Insert a record"""
        columns = ", ".join(kw.keys())
        placeholders = ", ".join("?" for _ in kw)
        values = tuple(kw.values())
        query = f"INSERT INTO {table} ({columns}) VALUES ({placeholders});"
        
        try:
            self.cursor.execute(query, values)
            self.connection.commit()
            return True
        except sqlite3.IntegrityError:
            print("Duplicate Entry: Try updating or deleting.")
            return False
        except sqlite3.Error as e:
            print(f"Error inserting data: {e}")
            return False
    
    def update(self, table: str, where: str, **kw: Any) -> bool:
        """Update records"""
        set_clause = ", ".join(f"{k} = ?" for k in kw.keys())
        values = tuple(kw.values())
        query = f"UPDATE {table} SET {set_clause} WHERE {where};"
        
        try:
            cur = self.cursor
            cur.execute(query, values)
            self.connection.commit()
            return cur.rowcount > 0
        except sqlite3.Error as e:
            print(f"Error updating data: {e}")
            return False
    
    def delete(self, table: str, where: str) -> bool:
        """Delete records"""
        query = f"DELETE FROM {table} WHERE {where};"
        
        try:
            cur = self.cursor
            cur.execute(query)
            self.connection.commit()
            return cur.rowcount > 0
        except sqlite3.Error as e:
            print(f"Error deleting data: {e}")
            return False
    
    def close(self) -> None:
        """Close connection (call on app exit)"""
        if self._connection:
            self._connection.close()
            self._connection = None
    
    @classmethod
    def close_all(cls) -> None:
        """Close all database connections"""
        for instance in cls._instances.values():
            instance.close()
        cls._instances.clear()


# Usage example:
if __name__ == "__main__":
    # Get singleton instance
    db = Database('commands')  # or whatever your db_key is
    
    # Use it
    db.insert('commands', alias='test', command='echo hello')
    results = db.find('commands', "alias='test'")
    
    # Same instance everywhere
    db2 = Database('commands')
    assert db is db2  # True - same instance
    
    # Close when done (optional, but good practice)
    Database.close_all()