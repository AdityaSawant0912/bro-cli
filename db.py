import sqlite3
import os
from constants import *

def getCursor(db: str):
    db_path = os.path.join(PATH_TO_DB, db + ".db")

    con = sqlite3.connect(db_path)
    cur = con.cursor()
    return con, cur

def checkDB(db_key: str) -> bool:
    if db_key not in default_dbs:
        raise ValueError(f"Invalid database key: {db_key}")

    db_name = default_dbs[db_key] + ".db"

    if not os.path.isfile(os.path.join(PATH_TO_DB, db_name)):
        print(f"Database '{db_name}' not found. Initializing...")
        init(db_key, default_schemas[db_key])

    return True



def init(db_key: str, schema_file: str) -> None:
    if db_key not in default_dbs:
        raise ValueError(f"Invalid database key: {db_key}")

    db_name = default_dbs[db_key]
    con, cur = getCursor(db_name)

    # Check if the database is empty
    cur.execute("SELECT name FROM sqlite_master WHERE type='table';")
    tables = cur.fetchall()

    path_to_schema = os.path.join(PATH_TO_DB, schema_file)
    if not tables:  # If no tables exist, initialize the database
        if not os.path.isfile(path_to_schema):
            raise FileNotFoundError(f"Schema file not found: {path_to_schema}")

        with open(path_to_schema, 'r') as f:
            schema_sql = f.read()

        cur.executescript(schema_sql)
        con.commit()
        print(f"Database '{db_name}' initialized successfully.")

    con.close()

def find(db: str, table: str, where: str, select:str ='*') -> list:
    if not checkDB(db):
        raise ValueError(f"Invalid database name: {db}")

    con, cur = getCursor(default_dbs[db])

    query = f"SELECT {select} FROM {table} WHERE {where};"

    try:
        cur.execute(query)
        return cur.fetchall()
    except sqlite3.Error as e:
        print(f"Error fetching data: {e}")
        return []
    finally:
        con.close()

def insert(db: str, table: str, **kw: list) -> bool:
    if not checkDB(db):
        raise ValueError(f"Invalid database name: {db}")

    con, cur = getCursor(default_dbs[db])

    columns = ", ".join(kw.keys())
    placeholders = ", ".join("?" for _ in kw)
    values = tuple(kw.values())

    query = f"INSERT INTO {table} ({columns}) VALUES ({placeholders});"

    try:
        cur.execute(query, values)
        con.commit()
        return True
    except sqlite3.IntegrityError:
        print(f"Duplicate Entry: Try updating or deleting.")
        return False
    except sqlite3.Error as e:
        print(f"Error inserting data: {e}")
        return False
    finally:
        con.close()

def update(db: str, table: str, where: str, **kw: list) -> bool:
    if not checkDB(db):
        raise ValueError(f"Invalid database name: {db}")

    con, cur = getCursor(default_dbs[db])

    set_clause = ", ".join(f"{k} = ?" for k in kw.keys())
    values = tuple(kw.values())

    query = f"UPDATE {table} SET {set_clause} WHERE {where};"

    try:
        cur.execute(query, values)
        con.commit()
        return cur.rowcount > 0
    except sqlite3.Error as e:
        print(f"Error updating data: {e}")
        return False
    finally:
        con.close()

def delete(db: str, table: str, where: str) -> bool:
    if not checkDB(db):
        raise ValueError(f"Invalid database name: {db}")

    con, cur = getCursor(default_dbs[db])

    query = f"DELETE FROM {table} WHERE {where};"

    try:
        cur.execute(query)
        con.commit()
        return cur.rowcount > 0
    except sqlite3.Error as e:
        print(f"Error deleting data: {e}")
        return False
    finally:
        con.close()
