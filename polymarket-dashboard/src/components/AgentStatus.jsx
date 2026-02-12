export default function AgentStatus({ state }) {
  const stateConfig = {
    Survival: { color: 'bg-red-500', bar: 'bg-red-500', pct: 10, desc: '<$15 — Preserve capital' },
    Defensive: { color: 'bg-orange-500', bar: 'bg-orange-500', pct: 30, desc: '$15-30 — High conviction' },
    Neutral: { color: 'bg-blue-500', bar: 'bg-blue-500', pct: 50, desc: '$30-60 — Balanced' },
    Aggressive: { color: 'bg-green-500', bar: 'bg-green-500', pct: 75, desc: '$60-120 — Compound' },
    Apex: { color: 'bg-purple-500', bar: 'bg-purple-500', pct: 100, desc: '>$120 — Max deploy' },
  }

  const hungerConfig = {
    Starving: { icon: '💀', color: 'text-red-500', desc: 'No profit 48h+' },
    Hungry: { icon: '🔥', color: 'text-orange-500', desc: 'No profit 24h' },
    Seeking: { icon: '🎯', color: 'text-yellow-400', desc: 'Below target' },
    Satisfied: { icon: '✅', color: 'text-green-400', desc: 'Target met' },
    Feasting: { icon: '🏆', color: 'text-purple-400', desc: 'Exceeding' },
  }

  const sc = stateConfig[state.agent_state] || stateConfig.Neutral
  const hc = hungerConfig[state.hunger_level] || hungerConfig.Seeking

  const formatTime = (iso) => {
    if (!iso) return 'Never'
    const d = new Date(iso)
    const diff = (Date.now() - d.getTime()) / 1000
    if (diff < 60) return `${Math.floor(diff)}s ago`
    if (diff < 3600) return `${Math.floor(diff / 60)}m ago`
    return `${Math.floor(diff / 3600)}h ago`
  }

  return (
    <div className="bg-[#1a1f2e] rounded-lg p-4 border border-gray-800">
      <h3 className="text-sm font-bold text-gray-300 mb-4">Agent Status</h3>

      {/* State Machine */}
      <div className="mb-4">
        <div className="flex justify-between items-center mb-2">
          <span className="text-xs text-gray-500">State Machine</span>
          <span className={`text-sm font-bold`} style={{color: sc.color.replace('bg-', '').includes('red') ? '#ef4444' : sc.color.replace('bg-', '').includes('orange') ? '#f97316' : sc.color.includes('blue') ? '#3b82f6' : sc.color.includes('green') ? '#10b981' : '#8b5cf6'}}>
            {state.agent_state}
          </span>
        </div>
        <div className="h-2 bg-gray-800 rounded-full overflow-hidden">
          <div className={`h-full ${sc.bar} transition-all duration-500 rounded-full`} style={{ width: `${sc.pct}%` }} />
        </div>
        <p className="text-[10px] text-gray-500 mt-1">{sc.desc}</p>
      </div>

      {/* Hunger Level */}
      <div className="mb-4">
        <div className="flex justify-between items-center mb-2">
          <span className="text-xs text-gray-500">Hunger Level</span>
          <span className={`text-sm font-bold ${hc.color}`}>
            {hc.icon} {state.hunger_level}
          </span>
        </div>
        <p className="text-[10px] text-gray-500">{hc.desc}</p>
      </div>

      {/* Timestamps */}
      <div className="space-y-2 border-t border-gray-800 pt-3">
        <div className="flex justify-between text-xs">
          <span className="text-gray-500">Last Scan</span>
          <span className="text-gray-300">{formatTime(state.last_scan)}</span>
        </div>
        <div className="flex justify-between text-xs">
          <span className="text-gray-500">Last Command</span>
          <span className="text-gray-300">{formatTime(state.last_command)}</span>
        </div>
        <div className="flex justify-between text-xs">
          <span className="text-gray-500">Last Trade</span>
          <span className="text-gray-300">{formatTime(state.last_trade_at)}</span>
        </div>
        <div className="flex justify-between text-xs">
          <span className="text-gray-500">Last Profit</span>
          <span className={state.last_profit_at ? 'text-green-400' : 'text-red-400'}>
            {formatTime(state.last_profit_at)}
          </span>
        </div>
      </div>

      {/* Errors */}
      {state.errors?.length > 0 && (
        <div className="mt-3 border-t border-gray-800 pt-3">
          <span className="text-xs text-red-400 font-bold">Recent Errors</span>
          <div className="mt-1 max-h-20 overflow-y-auto">
            {state.errors.slice(0, 3).map((e, i) => (
              <p key={i} className="text-[10px] text-red-300 truncate">{e}</p>
            ))}
          </div>
        </div>
      )}
    </div>
  )
}
