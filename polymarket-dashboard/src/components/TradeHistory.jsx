export default function TradeHistory({ trades }) {
  if (!trades || trades.length === 0) {
    return (
      <div className="bg-[#1a1f2e] rounded-lg p-4 border border-gray-800">
        <h3 className="text-sm font-bold text-gray-300 mb-3">Trade History</h3>
        <div className="text-center py-6 text-gray-500 text-sm">
          No trades yet
        </div>
      </div>
    )
  }

  const sorted = [...trades].reverse()

  return (
    <div className="bg-[#1a1f2e] rounded-lg p-4 border border-gray-800">
      <div className="flex justify-between items-center mb-3">
        <h3 className="text-sm font-bold text-gray-300">Trade History</h3>
        <span className="text-xs text-gray-500">{trades.length} trades</span>
      </div>

      <div className="overflow-x-auto max-h-[300px] overflow-y-auto">
        <table className="w-full text-xs">
          <thead className="sticky top-0 bg-[#1a1f2e]">
            <tr className="text-gray-500 border-b border-gray-800">
              <th className="py-2 text-left">Time</th>
              <th className="py-2 text-left">Market</th>
              <th className="py-2 text-right">Side</th>
              <th className="py-2 text-right">Price</th>
              <th className="py-2 text-right">Size</th>
              <th className="py-2 text-right">Edge</th>
              <th className="py-2 text-right">Conf</th>
              <th className="py-2 text-right">API $</th>
              <th className="py-2 text-right">Status</th>
              <th className="py-2 text-left pl-3">Reason</th>
            </tr>
          </thead>
          <tbody>
            {sorted.map((trade, i) => {
              const time = trade.timestamp
                ? new Date(trade.timestamp).toLocaleTimeString('en-US', {
                    hour: '2-digit', minute: '2-digit', second: '2-digit'
                  })
                : '—'

              const statusColor = {
                Pending: 'text-yellow-400',
                Matched: 'text-blue-400',
                Confirmed: 'text-green-400',
                Failed: 'text-red-400',
                Retrying: 'text-orange-400',
              }

              return (
                <tr key={trade.id || i} className="border-b border-gray-800/50 hover:bg-gray-800/30">
                  <td className="py-2 text-gray-500">{time}</td>
                  <td className="py-2 text-gray-300 max-w-[200px] truncate">
                    {trade.market_question || trade.market_id}
                  </td>
                  <td className="py-2 text-right">
                    <span className={trade.side === 'Buy' ? 'text-green-400' : 'text-red-400'}>
                      {trade.side === 'Buy' ? 'BUY' : 'SELL'}
                    </span>
                  </td>
                  <td className="py-2 text-right text-gray-300">{trade.price?.toFixed(3)}</td>
                  <td className="py-2 text-right text-gray-300">${trade.size?.toFixed(2)}</td>
                  <td className="py-2 text-right text-blue-400">{(trade.edge * 100)?.toFixed(0)}%</td>
                  <td className="py-2 text-right text-purple-400">{(trade.confidence * 100)?.toFixed(0)}%</td>
                  <td className="py-2 text-right text-gray-500">${trade.api_cost?.toFixed(4)}</td>
                  <td className={`py-2 text-right ${statusColor[trade.status] || 'text-gray-400'}`}>
                    {trade.status}
                  </td>
                  <td className="py-2 text-left pl-3 text-gray-500 max-w-[150px] truncate">
                    {trade.reason}
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
