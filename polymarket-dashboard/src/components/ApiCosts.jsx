export default function ApiCosts({ costs }) {
  const defaults = {
    input_tokens: 0,
    output_tokens: 0,
    cache_read_tokens: 0,
    cache_write_tokens: 0,
    total_cost_usd: 0,
    daily_cost_usd: 0,
    calls_today: 0,
    daily_budget: 0.50,
    cached_pct: 0,
    ...costs,
  }

  const {
    input_tokens, output_tokens, cache_read_tokens, cache_write_tokens,
    total_cost_usd, daily_cost_usd, calls_today, daily_budget, cached_pct,
  } = defaults

  const budgetPct = daily_budget > 0 ? Math.min((daily_cost_usd / daily_budget) * 100, 100) : 0
  const budgetColor = budgetPct > 80 ? 'bg-red-500' : budgetPct > 50 ? 'bg-yellow-500' : 'bg-green-500'

  const formatTokens = (n) => n >= 1_000_000 ? (n / 1_000_000).toFixed(2) + 'M' : n >= 1_000 ? (n / 1_000).toFixed(1) + 'K' : n.toString()

  const rows = [
    { label: 'Input Tokens', value: formatTokens(input_tokens), rate: '$5.00/MTok', cost: (input_tokens / 1_000_000 * 5).toFixed(4) },
    { label: 'Output Tokens', value: formatTokens(output_tokens), rate: '$25.00/MTok', cost: (output_tokens / 1_000_000 * 25).toFixed(4) },
    { label: 'Cache Read', value: formatTokens(cache_read_tokens), rate: '$0.50/MTok', cost: (cache_read_tokens / 1_000_000 * 0.5).toFixed(4), highlight: true },
    { label: 'Cache Write', value: formatTokens(cache_write_tokens), rate: '$6.25/MTok', cost: (cache_write_tokens / 1_000_000 * 6.25).toFixed(4) },
  ]

  return (
    <div className="bg-[#1a1f2e] rounded-lg p-4 border border-gray-800">
      <div className="flex justify-between items-center mb-3">
        <h3 className="text-sm font-bold text-gray-300">API Costs (Claude Opus 4.6)</h3>
        <span className="text-xs text-gray-500">{calls_today} calls today</span>
      </div>

      {/* Budget Bar */}
      <div className="mb-4">
        <div className="flex justify-between text-xs mb-1">
          <span className="text-gray-400">Daily Budget</span>
          <span className="text-gray-300">
            ${daily_cost_usd.toFixed(4)} / ${daily_budget.toFixed(2)}
          </span>
        </div>
        <div className="w-full h-2 bg-gray-800 rounded-full overflow-hidden">
          <div
            className={`h-full ${budgetColor} rounded-full transition-all duration-500`}
            style={{ width: `${budgetPct}%` }}
          />
        </div>
        <div className="flex justify-between text-[10px] mt-1">
          <span className="text-gray-500">{budgetPct.toFixed(0)}% used</span>
          <span className="text-gray-500">
            ${(daily_budget - daily_cost_usd).toFixed(4)} remaining
          </span>
        </div>
      </div>

      {/* Token Breakdown */}
      <table className="w-full text-xs mb-3">
        <thead>
          <tr className="text-gray-500 border-b border-gray-800">
            <th className="py-1 text-left">Type</th>
            <th className="py-1 text-right">Tokens</th>
            <th className="py-1 text-right">Rate</th>
            <th className="py-1 text-right">Cost</th>
          </tr>
        </thead>
        <tbody>
          {rows.map((r) => (
            <tr key={r.label} className="border-b border-gray-800/50">
              <td className={`py-1.5 ${r.highlight ? 'text-green-400' : 'text-gray-400'}`}>
                {r.label} {r.highlight && '✨'}
              </td>
              <td className="py-1.5 text-right text-gray-300 font-mono">{r.value}</td>
              <td className="py-1.5 text-right text-gray-500">{r.rate}</td>
              <td className="py-1.5 text-right text-gray-300">${r.cost}</td>
            </tr>
          ))}
        </tbody>
      </table>

      {/* Summary Stats */}
      <div className="grid grid-cols-3 gap-2 pt-2 border-t border-gray-800">
        <div className="text-center">
          <div className="text-[10px] text-gray-500">Total Cost</div>
          <div className="text-sm font-bold text-gray-200">${total_cost_usd.toFixed(4)}</div>
        </div>
        <div className="text-center">
          <div className="text-[10px] text-gray-500">Cache Hit</div>
          <div className="text-sm font-bold text-green-400">{cached_pct.toFixed(0)}%</div>
        </div>
        <div className="text-center">
          <div className="text-[10px] text-gray-500">Avg $/Call</div>
          <div className="text-sm font-bold text-gray-200">
            ${calls_today > 0 ? (daily_cost_usd / calls_today).toFixed(4) : '0.0000'}
          </div>
        </div>
      </div>
    </div>
  )
}
