export default function MetricCards({ state }) {
  const pnlColor = state.total_pnl >= 0 ? 'text-green-400' : 'text-red-400'
  const dailyColor = state.daily_pnl >= 0 ? 'text-green-400' : 'text-red-400'
  const pnlGlow = state.total_pnl >= 0 ? 'glow-green' : 'glow-red'

  const cards = [
    {
      label: 'Capital',
      value: `$${state.capital.toFixed(2)}`,
      sub: `Initial: $${state.initial_capital.toFixed(2)}`,
      color: 'text-white',
      glow: 'glow-blue',
    },
    {
      label: 'Total P&L',
      value: `${state.total_pnl >= 0 ? '+' : ''}$${state.total_pnl.toFixed(2)}`,
      sub: `${state.total_pnl_pct >= 0 ? '+' : ''}${state.total_pnl_pct.toFixed(1)}%`,
      color: pnlColor,
      glow: pnlGlow,
    },
    {
      label: 'Daily P&L',
      value: `${state.daily_pnl >= 0 ? '+' : ''}$${state.daily_pnl.toFixed(4)}`,
      sub: `Target: $${(state.capital * 0.005).toFixed(2)}/day`,
      color: dailyColor,
      glow: '',
    },
    {
      label: 'Win Rate',
      value: `${state.win_rate.toFixed(1)}%`,
      sub: `${state.winning_trades}/${state.total_trades} trades`,
      color: state.win_rate >= 55 ? 'text-green-400' : state.win_rate >= 45 ? 'text-yellow-400' : 'text-red-400',
      glow: '',
    },
    {
      label: 'Positions',
      value: String(state.position_count),
      sub: `Max: 5 | Markets: ${state.market_count}`,
      color: 'text-blue-400',
      glow: '',
    },
    {
      label: 'API Cost Today',
      value: `$${state.api_daily_cost?.toFixed(4) || '0.0000'}`,
      sub: `Budget: ${state.api_budget_used_pct?.toFixed(0) || 0}% used`,
      color: (state.api_budget_used_pct || 0) > 80 ? 'text-red-400' : 'text-green-400',
      glow: '',
    },
  ]

  return (
    <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-3">
      {cards.map((card, i) => (
        <div
          key={i}
          className={`bg-[#1a1f2e] rounded-lg p-4 border border-gray-800 ${card.glow}`}
        >
          <p className="text-[11px] text-gray-500 uppercase tracking-wider mb-1">
            {card.label}
          </p>
          <p className={`text-xl font-bold ${card.color}`}>
            {card.value}
          </p>
          <p className="text-[10px] text-gray-500 mt-1">{card.sub}</p>
        </div>
      ))}
    </div>
  )
}
