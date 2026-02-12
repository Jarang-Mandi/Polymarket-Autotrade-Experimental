export default function PositionsTable({ positions }) {
  if (!positions || positions.length === 0) {
    return (
      <div className="bg-[#1a1f2e] rounded-lg p-4 border border-gray-800">
        <h3 className="text-sm font-bold text-gray-300 mb-3">Open Positions</h3>
        <div className="text-center py-8 text-gray-500 text-sm">
          No open positions
          <p className="text-xs text-gray-600 mt-1">Agent is scanning for opportunities...</p>
        </div>
      </div>
    )
  }

  return (
    <div className="bg-[#1a1f2e] rounded-lg p-4 border border-gray-800">
      <div className="flex justify-between items-center mb-3">
        <h3 className="text-sm font-bold text-gray-300">Open Positions ({positions.length}/5)</h3>
        <span className="text-xs text-gray-500">
          Total Unrealized: {' '}
          <span className={positions.reduce((s, p) => s + (p.unrealized_pnl || 0), 0) >= 0 ? 'text-green-400' : 'text-red-400'}>
            ${positions.reduce((s, p) => s + (p.unrealized_pnl || 0), 0).toFixed(4)}
          </span>
        </span>
      </div>

      <div className="overflow-x-auto">
        <table className="w-full text-xs">
          <thead>
            <tr className="text-gray-500 border-b border-gray-800">
              <th className="py-2 text-left">Market</th>
              <th className="py-2 text-right">Side</th>
              <th className="py-2 text-right">Size</th>
              <th className="py-2 text-right">Entry</th>
              <th className="py-2 text-right">Current</th>
              <th className="py-2 text-right">P&L</th>
              <th className="py-2 text-right">P&L %</th>
            </tr>
          </thead>
          <tbody>
            {positions.map((pos, i) => {
              const pnl = pos.unrealized_pnl || 0
              const pnlPct = pos.cost_basis ? (pnl / pos.cost_basis * 100) : 0
              const pnlColor = pnl >= 0 ? 'text-green-400' : 'text-red-400'

              return (
                <tr key={pos.id || i} className="border-b border-gray-800/50 hover:bg-gray-800/30">
                  <td className="py-2 text-left max-w-[200px] truncate text-gray-300">
                    {pos.market_question || pos.market_id}
                  </td>
                  <td className="py-2 text-right">
                    <span className={pos.side === 'Buy' ? 'text-green-400' : 'text-red-400'}>
                      {pos.side === 'Buy' ? 'YES' : 'NO'}
                    </span>
                  </td>
                  <td className="py-2 text-right text-gray-300">${pos.size?.toFixed(2)}</td>
                  <td className="py-2 text-right text-gray-400">{pos.entry_price?.toFixed(3)}</td>
                  <td className="py-2 text-right text-gray-300">{pos.current_price?.toFixed(3)}</td>
                  <td className={`py-2 text-right font-bold ${pnlColor}`}>
                    {pnl >= 0 ? '+' : ''}{pnl.toFixed(4)}
                  </td>
                  <td className={`py-2 text-right ${pnlColor}`}>
                    {pnlPct >= 0 ? '+' : ''}{pnlPct.toFixed(1)}%
                  </td>
                </tr>
              )
            })}
          </tbody>
        </table>
      </div>
    </div>
  )
}
