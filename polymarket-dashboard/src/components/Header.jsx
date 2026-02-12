export default function Header({ state, connected }) {
  const stateColors = {
    Survival: 'text-red-500',
    Defensive: 'text-orange-500',
    Neutral: 'text-blue-400',
    Aggressive: 'text-green-400',
    Apex: 'text-purple-400',
  }

  return (
    <header className="max-w-[1600px] mx-auto mb-6 flex items-center justify-between">
      <div className="flex items-center gap-3">
        <span className="text-3xl">🦞</span>
        <div>
          <h1 className="text-xl font-bold text-white tracking-tight">
            Polymarket Agent
          </h1>
          <p className="text-xs text-gray-500">
            Autonomous Trading Engine v0.1
          </p>
        </div>
      </div>

      <div className="flex items-center gap-6 text-sm">
        <div className="flex items-center gap-2">
          <span className="text-gray-400">State:</span>
          <span className={`font-bold ${stateColors[state.agent_state] || 'text-gray-400'}`}>
            {state.agent_state}
          </span>
        </div>

        <div className="flex items-center gap-2">
          <span className="text-gray-400">Hunger:</span>
          <span className="font-bold text-yellow-400">
            {state.hunger_level}
          </span>
        </div>

        <div className="flex items-center gap-2">
          <div className={`pulse-dot ${connected ? 'bg-green-500' : 'bg-red-500'}`} />
          <span className={connected ? 'text-green-400' : 'text-red-400'}>
            {connected ? 'LIVE' : 'OFFLINE'}
          </span>
        </div>

        {state.engine_running && (
          <div className="flex items-center gap-2">
            <div className="pulse-dot bg-blue-500" />
            <span className="text-blue-400">ENGINE ON</span>
          </div>
        )}
      </div>
    </header>
  )
}
