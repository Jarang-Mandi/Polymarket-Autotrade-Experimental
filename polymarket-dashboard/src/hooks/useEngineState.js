import { useState, useEffect, useRef, useCallback } from 'react'

const WS_URL = `ws://${window.location.hostname}:3001/ws`
const API_BASE = '/api'

// Demo data for when engine is offline
const DEMO_STATE = {
  capital: 50.0, initial_capital: 50.0, total_pnl: 0.0, total_pnl_pct: 0.0,
  daily_pnl: 0.0, agent_state: 'Neutral', hunger_level: 'Seeking',
  win_rate: 0.0, total_trades: 0, winning_trades: 0, position_count: 0,
  market_count: 0, engine_running: false, last_scan: null, last_analysis: null,
  last_trade_at: null, last_profit_at: null, uptime_hours: 0,
  timestamp: new Date().toISOString(), api_budget_used_pct: 0,
  api_daily_cost: 0, api_total_cost: 0, errors: [],
  last_command: null,
}

export function useEngineState() {
  const [state, setState] = useState(DEMO_STATE)
  const [positions, setPositions] = useState([])
  const [trades, setTrades] = useState([])
  const [markets, setMarkets] = useState([])
  const [costs, setCosts] = useState(null)
  const [connected, setConnected] = useState(false)
  const [capitalHistory, setCapitalHistory] = useState([{ time: Date.now(), capital: 50 }])
  const [arbOpportunities, setArbOpportunities] = useState([])
  const [arbStats, setArbStats] = useState(null)
  const wsRef = useRef(null)

  // WebSocket connection
  useEffect(() => {
    function connect() {
      try {
        const ws = new WebSocket(WS_URL)
        wsRef.current = ws

        ws.onopen = () => {
          setConnected(true)
          console.log('WS connected')
        }

        ws.onmessage = (event) => {
          try {
            const msg = JSON.parse(event.data)
            if (msg.msg_type === 'state_update' && msg.payload) {
              const p = msg.payload
              setState(prev => ({
                ...prev,
                capital: p.capital,
                total_pnl: p.total_pnl,
                total_pnl_pct: p.total_pnl_pct,
                daily_pnl: p.daily_pnl,
                agent_state: p.agent_state,
                hunger_level: p.hunger_level,
                win_rate: p.win_rate,
                total_trades: p.total_trades,
                winning_trades: p.winning_trades,
                position_count: p.positions?.length || 0,
                engine_running: true,
                timestamp: p.timestamp,
              }))
              setPositions(p.positions || [])

              // Track capital history
              setCapitalHistory(prev => {
                const next = [...prev, { time: Date.now(), capital: p.capital }]
                return next.length > 200 ? next.slice(-200) : next
              })
            }
          } catch (e) {
            console.error('WS parse error:', e)
          }
        }

        ws.onclose = () => {
          setConnected(false)
          setTimeout(connect, 3000)
        }

        ws.onerror = () => {
          ws.close()
        }
      } catch (e) {
        setTimeout(connect, 3000)
      }
    }

    connect()
    return () => wsRef.current?.close()
  }, [])

  // Poll REST endpoints
  const fetchData = useCallback(async () => {
    try {
      const [stateRes, tradesRes, marketsRes, costsRes, arbRes, arbStatsRes] = await Promise.all([
        fetch(`${API_BASE}/state`).then(r => r.ok ? r.json() : null).catch(() => null),
        fetch(`${API_BASE}/trades`).then(r => r.ok ? r.json() : null).catch(() => null),
        fetch(`${API_BASE}/markets`).then(r => r.ok ? r.json() : null).catch(() => null),
        fetch(`${API_BASE}/costs`).then(r => r.ok ? r.json() : null).catch(() => null),
        fetch(`${API_BASE}/arb-opportunities`).then(r => r.ok ? r.json() : null).catch(() => null),
        fetch(`${API_BASE}/arb-stats`).then(r => r.ok ? r.json() : null).catch(() => null),
      ])
      if (stateRes) setState(prev => ({ ...prev, ...stateRes }))
      if (tradesRes) setTrades(tradesRes)
      if (marketsRes) setMarkets(marketsRes)
      if (costsRes) setCosts(costsRes)
      if (arbRes) setArbOpportunities(arbRes)
      if (arbStatsRes) setArbStats(arbStatsRes)
    } catch (e) { /* offline */ }
  }, [])

  useEffect(() => {
    fetchData()
    const interval = setInterval(fetchData, 10000)
    return () => clearInterval(interval)
  }, [fetchData])

  return { state, positions, trades, markets, costs, connected, capitalHistory, arbOpportunities, arbStats }
}
