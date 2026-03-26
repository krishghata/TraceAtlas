/**
 * Parse raw `ping` command output into structured per-packet data + summary stats.
 * Handles both Windows (`ping -n`) and Unix (`ping -c`) output formats.
 */
export function parsePingOutput(output, isWin) {
  const lines = output.split('\n')
  const packets = []

  for (const line of lines) {
    const t = line.trim()
    if (!t) continue

    if (isWin) {
      // "Reply from 8.8.8.8: bytes=32 time=12ms TTL=117"
      // "Reply from 8.8.8.8: bytes=32 time<1ms TTL=64"
      // "Request timed out."
      // "Destination host unreachable."
      const m = t.match(/time[=<](\d+)ms/i)
      if (m) {
        packets.push({ rtt: parseInt(m[1], 10), dropped: false })
      } else if (
        t.toLowerCase().includes('timed out') ||
        t.toLowerCase().includes('unreachable') ||
        t.toLowerCase().includes('could not find host') ||
        t.toLowerCase().includes('transmit failed')
      ) {
        packets.push({ rtt: null, dropped: true })
      }
    } else {
      // "64 bytes from 8.8.8.8: icmp_seq=1 ttl=117 time=12.3 ms"
      // "Request timeout for icmp_seq 1"
      // "ping: cannot resolve foo: Name or service not known"
      const m = t.match(/time[=<](\d+\.?\d*)\s*ms/i)
      if (m) {
        packets.push({ rtt: parseFloat(m[1]), dropped: false })
      } else if (
        t.toLowerCase().includes('timeout') ||
        t.toLowerCase().includes('unreachable') ||
        t.toLowerCase().includes('no route') ||
        t.toLowerCase().includes('cannot resolve') ||
        t.toLowerCase().includes('unknown host')
      ) {
        packets.push({ rtt: null, dropped: true })
      }
    }
  }

  const rtts    = packets.filter(p => !p.dropped).map(p => p.rtt)
  const dropped = packets.filter(p => p.dropped).length

  const stats = {
    sent:     packets.length,
    received: rtts.length,
    loss:     packets.length > 0 ? Math.round((dropped / packets.length) * 100) : 0,
    min:      rtts.length > 0 ? Math.min(...rtts) : null,
    max:      rtts.length > 0 ? Math.max(...rtts) : null,
    avg:      rtts.length > 0 ? Math.round(rtts.reduce((a, b) => a + b, 0) / rtts.length) : null,
    jitter:   rtts.length > 1 ? calcJitter(rtts) : null,
  }

  return { packets, stats }
}

function calcJitter(rtts) {
  let sum = 0
  for (let i = 1; i < rtts.length; i++) sum += Math.abs(rtts[i] - rtts[i - 1])
  return Math.round(sum / (rtts.length - 1))
}

/** Map an RTT value (ms) to a color. null = timeout = red. */
export function rttColor(rtt) {
  if (rtt === null) return '#ef4444'
  if (rtt < 50)    return '#22c55e'
  if (rtt < 150)   return '#f59e0b'
  if (rtt < 300)   return '#f97316'
  return '#ef4444'
}
