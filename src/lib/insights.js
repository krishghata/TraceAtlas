function haversine(lat1, lon1, lat2, lon2) {
  const R    = 6371
  const dLat = (lat2 - lat1) * Math.PI / 180
  const dLon = (lon2 - lon1) * Math.PI / 180
  const a    =
    Math.sin(dLat / 2) ** 2 +
    Math.cos(lat1 * Math.PI / 180) * Math.cos(lat2 * Math.PI / 180) * Math.sin(dLon / 2) ** 2
  return R * 2 * Math.atan2(Math.sqrt(a), Math.sqrt(1 - a))
}

export function generateInsights(hops) {
  const countries = [...new Set(hops.map(h => h.country).filter(Boolean))]

  // Country transitions
  const transitions = []
  for (let i = 1; i < hops.length; i++) {
    if (hops[i].country && hops[i - 1].country && hops[i].country !== hops[i - 1].country) {
      transitions.push(`${hops[i - 1].country} → ${hops[i].country}`)
    }
  }

  // Total distance
  let totalDistance = 0
  for (let i = 1; i < hops.length; i++) {
    if (hops[i].lat != null && hops[i - 1].lat != null) {
      totalDistance += haversine(hops[i - 1].lat, hops[i - 1].lon, hops[i].lat, hops[i].lon)
    }
  }
  totalDistance = Math.round(totalDistance)

  // Distance fun fact
  let distanceFact = null
  if (totalDistance > 15000)     distanceFact = `~${totalDistance.toLocaleString()} km — more than a third of Earth's circumference!`
  else if (totalDistance > 5000) distanceFact = `~${totalDistance.toLocaleString()} km — longer than a transatlantic flight!`
  else if (totalDistance > 1000) distanceFact = `~${totalDistance.toLocaleString()} km — roughly ${Math.round(totalDistance / 400)} hours by car.`
  else                           distanceFact = `~${totalDistance.toLocaleString()} km — a relatively local route.`

  // Latency spike
  let latency = null
  for (let i = 1; i < hops.length; i++) {
    if (hops[i].latency != null && hops[i - 1].latency != null) {
      const diff = hops[i].latency - hops[i - 1].latency
      if (diff > 50) {
        latency = `Spike of +${diff}ms at hop ${hops[i].hop} (${hops[i - 1].latency}ms → ${hops[i].latency}ms)`
        break
      }
    }
  }

  // Bottleneck
  let bottleneck = null
  const withLatency = hops.filter(h => h.latency != null)
  if (withLatency.length) {
    const worst = withLatency.reduce((a, b) => b.latency > a.latency ? b : a)
    if (worst.latency > 10) {
      bottleneck = `Slowest hop: #${worst.hop} in ${worst.country || 'unknown'} at ${worst.latency}ms`
    }
  }

  // Ocean crossing heuristic
  let ocean = null
  for (let i = 1; i < hops.length; i++) {
    if (hops[i].country && hops[i - 1].country && hops[i].country !== hops[i - 1].country) {
      const dist = haversine(hops[i - 1].lat, hops[i - 1].lon, hops[i].lat, hops[i].lon)
      if (dist > 3000) {
        ocean = `Likely crossed an ocean between ${hops[i - 1].country} and ${hops[i].country} (~${Math.round(dist).toLocaleString()} km)`
        break
      }
    }
  }

  const story = countries.length > 1
    ? `Your data crossed ${countries.length} countries (${countries.join(' → ')}) in ${hops.length} hops covering ~${totalDistance.toLocaleString()} km`
    : `Your data traveled ${hops.length} hops in ${countries[0] || 'unknown'} covering ~${totalDistance.toLocaleString()} km`

  return { story, transitions, latency, ocean, bottleneck, distanceFact, totalDistance, countries }
}
