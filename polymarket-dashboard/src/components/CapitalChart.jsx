import { LineChart, Line, XAxis, YAxis, Tooltip, ResponsiveContainer, ReferenceLine } from 'recharts'

export default function CapitalChart({ data, capital }) {
  const formatted = data.map(d => ({
    ...d,
    time: new Date(d.time).toLocaleTimeString('en-US', { hour: '2-digit', minute: '2-digit' }),
  }))

  const min = Math.min(...data.map(d => d.capital)) * 0.98
  const max = Math.max(...data.map(d => d.capital)) * 1.02

  return (
    <div className="bg-[#1a1f2e] rounded-lg p-4 border border-gray-800 h-[300px]">
      <div className="flex justify-between items-center mb-3">
        <h3 className="text-sm font-bold text-gray-300">Capital Curve</h3>
        <span className={`text-lg font-bold ${capital >= 50 ? 'text-green-400' : 'text-red-400'}`}>
          ${capital.toFixed(2)}
        </span>
      </div>

      <ResponsiveContainer width="100%" height="85%">
        <LineChart data={formatted}>
          <XAxis
            dataKey="time"
            tick={{ fill: '#6b7280', fontSize: 10 }}
            interval="preserveStartEnd"
          />
          <YAxis
            domain={[min, max]}
            tick={{ fill: '#6b7280', fontSize: 10 }}
            tickFormatter={v => `$${v.toFixed(0)}`}
          />
          <Tooltip
            contentStyle={{
              background: '#1a1f2e',
              border: '1px solid #374151',
              borderRadius: '8px',
              fontSize: '12px',
            }}
            formatter={v => [`$${Number(v).toFixed(2)}`, 'Capital']}
          />
          <ReferenceLine y={50} stroke="#374151" strokeDasharray="3 3" />
          <Line
            type="monotone"
            dataKey="capital"
            stroke={capital >= 50 ? '#10b981' : '#ef4444'}
            strokeWidth={2}
            dot={false}
            animationDuration={300}
          />
        </LineChart>
      </ResponsiveContainer>
    </div>
  )
}
