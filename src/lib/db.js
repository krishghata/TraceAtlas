import Database from '@tauri-apps/plugin-sql'

let _dbPromise = null

export function getDb() {
  if (!_dbPromise) {
    _dbPromise = Database.load('sqlite:traceatlas.db').then(async db => {
      await db.execute(`
        CREATE TABLE IF NOT EXISTS ip_cache (
          ip         TEXT PRIMARY KEY,
          lat        REAL,
          lon        REAL,
          country    TEXT,
          org        TEXT,
          updated_at INTEGER
        )
      `)
      await db.execute(`
        CREATE TABLE IF NOT EXISTS trace_cache (
          target     TEXT PRIMARY KEY,
          result     TEXT,
          updated_at INTEGER
        )
      `)
      return db
    })
  }
  return _dbPromise
}
