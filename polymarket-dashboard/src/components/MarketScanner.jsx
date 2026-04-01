import { useState } from 'react'
import { ArrowUpDown } from 'lucide-react'

export default function MarketScanner({ markets }) {
  const [sortKey, setSortKey] = useState('volume_24h')
  const [sortAsc, setSortAsc] = useState(false)

  if (!markets || markets.length === 0) {
    return (
      <div className="bg-[#1a1f2e] rounded-lg p-4 border border-gray-800">
        <h3 className="text-sm font-bold text-gray-300 mb-3">Market Scanner</h3>
        <div className="text-center py-6 text-gray-500 text-sm">
          Scanning markets...
        </div>
      </div>
    )
  }

  const handleSort = (key) => {
    if (sortKey === key) setSortAsc(!sortAsc)
    else { setSortKey(key); setSortAsc(false) }
  }

  const sorted = [...markets].sort((a, b) => {
    const va = a[sortKey] ?? 0
    const vb = b[sortKey] ?? 0
    return sortAsc ? va - vb : vb - va
  })

  const SortHeader = ({ label, field, align = 'right' }) => (
    <th
      className={`py-2 cursor-pointer hover:text-gray-300 transition ${align === 'left' ? 'text-left' : 'text-right'}`}
      onClick={() => handleSort(field)}
    >
      <span className="inline-flex items-center gap-1">
        {label}
        {sortKey === field && <ArrowUpDown className="w-3 h-3" />}
      </span>
    </th>
  )

  return (
    <div className="bg-[#1a1f2e] rounded-lg p-4 border border-gray-800">
      <div className="flex justify-between items-center mb-3">
        <h3 className="text-sm font-bold text-gray-300">Market Scanner</h3>
        <span className="text-xs text-gray-500">{markets.length} markets</span>
      </div>

      <div className="overflow-x-auto max-h-[300px] overflow-y-auto">
        <table className="w-full text-xs">
          <thead className="sticky top-0 bg-[#1a1f2e] text-gray-500 border-b border-gray-800">
            <tr>
              <th className="py-2 text-left">Market</th>
              <SortHeader label="YES ¢" field="yes_price" />
              <SortHeader label="Spread" field="spread" />
              <SortHeader label="Vol 24h" field="volume_24h" />
              <SortHeader label="Liquidity" field="liquidity" />
              <th className="py-2 text-right">Category</th>
              <th className="py-2 text-right">Ends</th>
            </tr>
          </thead>
          <tbody>
            {sorted.map((m, i) => {
              const yesPrice = m.yes_price ?? 0
              const spread = m.spread ?? 0
              const vol = m.volume_24h ?? m.volume24hr ?? 0
              const liq = m.liquidity ?? 0

              const endDateValue = m.end_date_iso ?? m.end_date
              const end = endDateValue
                ? new Date(endDateValue).toLocaleDateString('en-US', { month: 'short', day: 'numeric' })
                : '—'

              const priceColor = yesPrice >= 0.9 || yesPrice <= 0.1
                ? 'text-gray-500'
                : yesPrice >= 0.7
                  ? 'text-green-400'
                  : yesPrice <= 0.3
                    ? 'text-red-400'
                    : 'text-yellow-400'

              return (
                <tr key={m.condition_id || i} className="border-b border-gray-800/50 hover:bg-gray-800/30">
                  <td className="py-2 text-gray-300 max-w-[250px] truncate">
                    {m.question || m.title}
                  </td>
                  <td className={`py-2 text-right font-mono ${priceColor}`}>
                    {(yesPrice * 100)?.toFixed(0)}¢
                  </td>
                  <td className="py-2 text-right text-gray-400">
                    {(spread * 100)?.toFixed(1)}¢
                  </td>
                  <td className="py-2 text-right text-blue-400">
                    ${vol >= 1000 ? (vol / 1000).toFixed(1) + 'k' : vol.toFixed(0)}
                  </td>
                  <td className="py-2 text-right text-purple-400">
                    ${liq >= 1000 ? (liq / 1000).toFixed(1) + 'k' : liq.toFixed(0)}
                  </td>
                  <td className="py-2 text-right text-gray-500">{m.category || '—'}</td>
                  <td className="py-2 text-right text-gray-500">{end}</td>
                </tr>
              )
            })}
          </tbody>
        </table>
      </div>
    </div>
  )
}
