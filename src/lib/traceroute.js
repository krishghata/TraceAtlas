/** Parse raw traceroute/tracert output into structured hops. */
export function parseTraceroute(output, isWin) {
  const hops = []

  for (const line of output.split('\n')) {
    if (isWin) {
      // Windows tracert:  1    <1 ms    <1 ms    <1 ms  8.8.8.8
      const ipMatch = line.match(/^\s*(\d+).*?((?:\d{1,3}\.){3}\d{1,3})\s*$/)
      if (!ipMatch) continue
      const latencies = [...line.matchAll(/<?\s*(\d+)\s*ms/gi)].map(m => parseInt(m[1]))
      hops.push({
        hop: parseInt(ipMatch[1]),
        ip: ipMatch[2],
        latency: latencies.length
          ? Math.round(latencies.reduce((a, b) => a + b) / latencies.length)
          : null
      })
    } else {
      // Linux/macOS:  1  8.8.8.8  1.123 ms
      const match = line.match(/^\s*(\d+)\s+((?:\d{1,3}\.){3}\d{1,3})\s+([\d.]+)\s*ms/)
      if (!match) continue
      hops.push({ hop: parseInt(match[1]), ip: match[2], latency: Math.round(parseFloat(match[3])) })
    }
  }

  return hops
}

export function isPrivateIP(ip) {
  const p = ip.split('.').map(Number)
  return (
    p[0] === 10 ||
    p[0] === 127 ||
    (p[0] === 172 && p[1] >= 16 && p[1] <= 31) ||
    (p[0] === 192 && p[1] === 168)
  )
}

export function deduplicateHops(hops) {
  const seen = new Set()
  return hops.filter(h => {
    if (seen.has(h.ip)) return false
    seen.add(h.ip)
    return true
  })
}
