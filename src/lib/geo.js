import { getDb } from './db.js'

const CACHE_TTL = 30 * 24 * 60 * 60 * 1000  // 30 days

/**
 * Batch resolve geo for an array of IPs.
 * Strategy:
 *   1. SQLite cache (fresh < 30 days) → use it
 *   2. ip-api.com batch call for misses/stale → cache result in SQLite
 *   3. Stale cache as last resort if API fails
 *
 * Note: geoip-lite is Node.js-only and cannot run in the WebView renderer.
 * The SQLite cache (persistent across app sessions) acts as the local DB.
 * ip-api.com is only called for new/expired IPs, keeping API usage minimal.
 */
export async function batchResolveGeo(ips) {
  const db      = await getDb()
  const results = new Map()
  const needsAPI = []

  for (const ip of ips) {
    const rows   = await db.select('SELECT * FROM ip_cache WHERE ip = ?', [ip])
    const cached = rows[0]

    if (cached && (Date.now() - cached.updated_at) < CACHE_TTL) {
      results.set(ip, { lat: cached.lat, lon: cached.lon, country: cached.country, org: cached.org })
      continue
    }

    needsAPI.push({ ip, staleCached: cached || null })
  }

  if (needsAPI.length > 0) {
    try {
      const res = await fetch(
        'http://ip-api.com/batch?fields=query,lat,lon,countryCode,org,status',
        {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(needsAPI.map(x => ({ query: x.ip })))
        }
      )
      const apiData = await res.json()

      for (const d of apiData) {
        const item = needsAPI.find(x => x.ip === d.query)
        if (d.status === 'success') {
          await db.execute(
            'INSERT OR REPLACE INTO ip_cache (ip, lat, lon, country, org, updated_at) VALUES (?, ?, ?, ?, ?, ?)',
            [d.query, d.lat, d.lon, d.countryCode, d.org, Date.now()]
          )
          results.set(d.query, { lat: d.lat, lon: d.lon, country: d.countryCode, org: d.org })
        } else if (item?.staleCached) {
          // API failed for this IP — fall back to stale cache
          const c = item.staleCached
          results.set(d.query, { lat: c.lat, lon: c.lon, country: c.country, org: c.org })
        }
      }
    } catch (_) {
      // Entire batch failed — use stale cache for all pending
      for (const item of needsAPI) {
        if (!results.has(item.ip) && item.staleCached) {
          const c = item.staleCached
          results.set(item.ip, { lat: c.lat, lon: c.lon, country: c.country, org: c.org })
        }
      }
    }
  }

  return results
}

/**
 * Get the user's own public IP and location.
 * Since this is a desktop app the machine running the app IS the user,
 * so ip-api.com/json (no IP arg) returns the correct source location.
 */
export async function getSourceLocation() {
  try {
    const res = await fetch('http://ip-api.com/json?fields=query,lat,lon,countryCode,status')
    const d   = await res.json()
    if (d.status === 'success') {
      return { ip: d.query, lat: d.lat, lon: d.lon, country: d.countryCode }
    }
  } catch (_) {}
  return null
}
