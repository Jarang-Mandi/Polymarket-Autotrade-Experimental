import { useEngineState } from './hooks/useEngineState'
import Header from './components/Header'
import MetricCards from './components/MetricCards'
import CapitalChart from './components/CapitalChart'
import PositionsTable from './components/PositionsTable'
import TradeHistory from './components/TradeHistory'
import MarketScanner from './components/MarketScanner'
import ApiCosts from './components/ApiCosts'
import AgentStatus from './components/AgentStatus'

export default function App() {
  const { state, positions, trades, markets, costs, connected, capitalHistory } = useEngineState()

  return (
    <div className="min-h-screen bg-[#0a0e17] p-4">
      <Header state={state} connected={connected} />

      <div className="max-w-[1600px] mx-auto space-y-4">
        <MetricCards state={state} />

        <div className="grid grid-cols-1 lg:grid-cols-3 gap-4">
          <div className="lg:col-span-2">
            <CapitalChart data={capitalHistory} capital={state.capital} />
          </div>
          <AgentStatus state={state} />
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
          <PositionsTable positions={positions} />
          <ApiCosts costs={costs} state={state} />
        </div>

        <TradeHistory trades={trades} />
        <MarketScanner markets={markets} />
      </div>

      <footer className="text-center text-gray-600 text-xs mt-8 pb-4">
        Polymarket Autonomous Agent — Powered by Claude Opus 4.6 + OpenClaw
      </footer>
    </div>
  )
}
