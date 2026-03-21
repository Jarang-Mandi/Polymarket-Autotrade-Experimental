import { useState } from 'react'
import { Zap, TrendingUp, Shield, Clock, ChevronDown, ChevronRight } from 'lucide-react'

const TYPE_COLORS = {
  'Binary Arb': 'text-green-400',
  'Multi-Outcome Under': 'text-blue-400',
  'Multi-Outcome Over': 'text-purple-400',
  'Spread Capture': 'text-yellow-400',
}

const TYPE_ICONS = {
  'Binary Arb': Shield,
  'Multi-Outcome Under': TrendingUp,
  'Multi-Outcome Over': TrendingUp,
  'Spread Capture': Zap,
}

function formatTime(iso) {
  if (!iso) return '—'
  const d = new Date(iso)
  const now = new Date()
  const diffMs = now - d
  const mins = Math.floor(diffMs / 60000)
  if (mins < 1) return 'just now'
  if (mins < 60) return `${mins}m ago`
  return `${Math.floor(mins / 60)}h ago`
}

function ArbOpportunity({ opp, expanded, onToggle }) {
  const Icon = TYPE_ICONS[opp.type] || Zap
  const color = TYPE_COLORS[opp.type] || 'text-gray-400'

  return (
    <div className="border border-gray-800 rounded-md overflow-hidden">
      <div
        className="flex items-center justify-between p-3 cursor-pointer hover:bg-[#252b3d] transition"
        onClick={onToggle}
      >
        <div className="flex items-center gap-2">
          {expanded ? <ChevronDown className="w-3 h-3 text-gray-500" /> : <ChevronRight className="w-3 h-3 text-gray-500" />}
          <Icon className={`w-4 h-4 ${color}`} />
          <span className={`text-xs font-semibold ${color}`}>{opp.type}</span>
          {opp.guaranteed && (
            <span className="text-[10px] bg-green-900/40 text-green-400 px-1.5 py-0.5 rounded">
              GUARANTEED
            </span>
          )}
        </div>
        <div className="flex items-center gap-4 text-xs">
          <span className="text-gray-400">{opp.legs} leg{opp.legs > 1 ? 's' : ''}</span>
          <span className="text-green-400 font-mono font-bold">
            +{opp.profit_pct.toFixed(2)}%
          </span>
          <span className="text-gray-300 font-mono">${opp.profit.toFixed(4)}/unit</span>
          <span className="text-gray-500">{formatTime(opp.detected_at)}</span>
        </div>
      </div>

      {expanded && (
        <div className="bg-[#0f1320] px-3 pb-3 space-y-2">
          <div className="grid grid-cols-3 gap-2 text-xs">
            <div>
              <span className="text-gray-500">Total Cost:</span>{' '}
              <span className="text-gray-300 font-mono">${opp.total_cost.toFixed(4)}</span>
            </div>
            <div>
              <span className="text-gray-500">Payout:</span>{' '}
              <span className="text-gray-300 font-mono">${opp.guaranteed_payout.toFixed(4)}</span>
            </div>
            <div>
              <span className="text-gray-500">Liquidity:</span>{' '}
              <span className="text-gray-300 font-mono">{opp.liquidity_score.toFixed(1)}/10</span>
            </div>
          </div>

          {opp.event_slug && (
            <div className="text-xs text-gray-500">
              Event: <span className="text-gray-400">{opp.event_slug}</span>
            </div>
          )}

          {opp.leg_details && opp.leg_details.length > 0 && (
            <div className="space-y-1">
              <div className="text-[10px] text-gray-500 uppercase tracking-wider">Legs</div>
              {opp.leg_details.map((leg, i) => (
                <div key={i} className="flex items-center gap-2 text-xs bg-[#1a1f2e] rounded px-2 py-1">
                  <span className={leg.side === 'Buy' ? 'text-green-400' : 'text-red-400'}>
                    {leg.side}
                  </span>
                  <span className="text-gray-300 font-mono">${leg.price.toFixed(3)}</span>
                  <span className="text-gray-500 truncate flex-1">{leg.question}</span>
                  {leg.neg_risk && (
                    <span className="text-[9px] bg-purple-900/30 text-purple-400 px-1 rounded">neg-risk</span>
                  )}
                </div>
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  )
}

export default function ArbitragePanel({ opportunities, arbStats }) {
  const [expandedId, setExpandedId] = useState(null)

  const stats = arbStats || {}
  const opps = opportunities || []

  const guaranteedCount = opps.filter(o => o.guaranteed).length
  const spreadCount = opps.filter(o => !o.guaranteed).length

  return (
    <div className="bg-[#1a1f2e] rounded-lg p-4 border border-gray-800">
      <div className="flex justify-between items-center mb-3">
        <div className="flex items-center gap-2">
          <Zap className="w-4 h-4 text-yellow-400" />
          <h3 className="text-sm font-bold text-gray-300">Arbitrage Scanner</h3>
          <span className={`text-[10px] px-1.5 py-0.5 rounded ${
            stats.scanner_enabled !== false
              ? 'bg-green-900/40 text-green-400'
              : 'bg-red-900/40 text-red-400'
          }`}>
            {stats.scanner_enabled !== false ? 'ACTIVE' : 'OFF'}
          </span>
        </div>
        <div className="flex items-center gap-3 text-xs text-gray-500">
          {stats.last_scan_at && (
            <span className="flex items-center gap-1">
              <Clock className="w-3 h-3" />
              {formatTime(stats.last_scan_at)}
            </span>
          )}
          <span>{opps.length} found</span>
        </div>
      </div>

      {/* Stats bar */}
      <div className="grid grid-cols-5 gap-2 mb-3 text-xs">
        <div className="bg-[#0f1320] rounded p-2 text-center">
          <div className="text-gray-500">Found</div>
          <div className="text-gray-200 font-mono">{stats.opportunities_found ?? 0}</div>
        </div>
        <div className="bg-[#0f1320] rounded p-2 text-center">
          <div className="text-gray-500">Executed</div>
          <div className="text-green-400 font-mono">{stats.opportunities_executed ?? 0}</div>
        </div>
        <div className="bg-[#0f1320] rounded p-2 text-center">
          <div className="text-gray-500">Profit</div>
          <div className="text-green-400 font-mono">${(stats.total_profit ?? 0).toFixed(2)}</div>
        </div>
        <div className="bg-[#0f1320] rounded p-2 text-center">
          <div className="text-gray-500">Partials</div>
          <div className="text-yellow-400 font-mono">{stats.partial_fills ?? 0}</div>
        </div>
        <div className="bg-[#0f1320] rounded p-2 text-center">
          <div className="text-gray-500">Active</div>
          <div className="text-blue-400 font-mono">{stats.active_arb_count ?? 0}</div>
        </div>
      </div>

      {/* Type summary */}
      {opps.length > 0 && (
        <div className="flex gap-2 mb-3 text-xs">
          {guaranteedCount > 0 && (
            <span className="bg-green-900/20 text-green-400 px-2 py-1 rounded">
              {guaranteedCount} guaranteed
            </span>
          )}
          {spreadCount > 0 && (
            <span className="bg-yellow-900/20 text-yellow-400 px-2 py-1 rounded">
              {spreadCount} spread
            </span>
          )}
        </div>
      )}

      {/* Opportunities list */}
      {opps.length === 0 ? (
        <div className="text-center py-6 text-gray-500 text-sm">
          No arbitrage opportunities detected yet.
          <div className="text-xs mt-1 text-gray-600">Scanner checks every 60 seconds</div>
        </div>
      ) : (
        <div className="space-y-1 max-h-[400px] overflow-y-auto">
          {opps.map(opp => (
            <ArbOpportunity
              key={opp.id}
              opp={opp}
              expanded={expandedId === opp.id}
              onToggle={() => setExpandedId(expandedId === opp.id ? null : opp.id)}
            />
          ))}
        </div>
      )}
    </div>
  )
}
