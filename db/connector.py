import sqlite3





def getCursor(db: str):
    con = sqlite3.connect(db + ".db")
    cur = con.cursor()
    return cur
    
def init() -> None:
    pass