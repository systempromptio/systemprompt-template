import type { Artifact } from '@/types/artifact'
import type { ChartHints } from '@/types/artifacts'
import { extractChartData, unwrapExtraction } from '@/lib/artifacts'
import { BarChart, Bar, LineChart, Line, AreaChart, Area, PieChart, Pie, Cell, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer } from 'recharts'

interface ChartRendererProps {
  artifact: Artifact
  hints: ChartHints
}

const COLORS = [
  'hsl(28, 91%, 60%)',  // primary
  'hsl(28, 91%, 70%)',  // secondary
  '#F1C40F',            // yellow
  '#2ECC71',            // green
  '#E67E22',            // orange
  '#EC7063',            // red
  '#48C9B0',            // teal
  '#FFFFFF',            // white
]

export function ChartRenderer({ artifact, hints }: ChartRendererProps) {
  const chartDataResult = extractChartData(artifact)
  const chartData = unwrapExtraction(chartDataResult)

  if (!chartData) {
    return <div className="text-secondary text-center py-8">Invalid chart data</div>
  }

  const chartType = hints.chart_type || 'bar'

  const transformedData = chartData.labels.map((label: string, i: number) => {
    const dataPoint: Record<string, string | number> = { name: label }
    chartData.datasets.forEach((dataset, datasetIdx: number) => {
      dataPoint[dataset.label || `series${datasetIdx}`] = dataset.data[i] || 0
    })
    return dataPoint
  })

  const dataKeys = chartData.datasets.map((ds, idx: number) => ds.label || `series${idx}`)

  const renderChart = () => {
    switch (chartType) {
      case 'bar':
        return (
          <BarChart data={transformedData}>
            <CartesianGrid strokeDasharray="3 3" />
            <XAxis dataKey="name" />
            <YAxis />
            <Tooltip />
            <Legend />
            {dataKeys.map((key: string, idx: number) => (
              <Bar
                key={key}
                dataKey={key}
                fill={chartData.datasets[idx]?.color || COLORS[idx % COLORS.length]}
              />
            ))}
          </BarChart>
        )

      case 'line':
        return (
          <LineChart data={transformedData}>
            <CartesianGrid strokeDasharray="3 3" />
            <XAxis dataKey="name" />
            <YAxis />
            <Tooltip />
            <Legend />
            {dataKeys.map((key: string, idx: number) => (
              <Line
                key={key}
                type="monotone"
                dataKey={key}
                stroke={chartData.datasets[idx]?.color || COLORS[idx % COLORS.length]}
                strokeWidth={2}
              />
            ))}
          </LineChart>
        )

      case 'area':
        return (
          <AreaChart data={transformedData}>
            <CartesianGrid strokeDasharray="3 3" />
            <XAxis dataKey="name" />
            <YAxis />
            <Tooltip />
            <Legend />
            {dataKeys.map((key: string, idx: number) => (
              <Area
                key={key}
                type="monotone"
                dataKey={key}
                fill={chartData.datasets[idx]?.color || COLORS[idx % COLORS.length]}
                stroke={chartData.datasets[idx]?.color || COLORS[idx % COLORS.length]}
              />
            ))}
          </AreaChart>
        )

      case 'pie':
        const pieData = transformedData.map((item: Record<string, string | number>) => ({
          name: item.name,
          value: Number(item[dataKeys[0]] || 0),
        }))
        return (
          <PieChart>
            <Pie
              data={pieData}
              cx="50%"
              cy="50%"
              labelLine={false}
              label={(entry) =>
                `${entry.name || ''} ${(Number(entry.percent || 0) * 100).toFixed(0)}%`
              }
              outerRadius={80}
              fill="#8884d8"
              dataKey="value"
            >
              {pieData.map((_: unknown, index: number) => (
                <Cell key={`cell-${index}`} fill={COLORS[index % COLORS.length]} />
              ))}
            </Pie>
            <Tooltip />
          </PieChart>
        )

      default:
        return (
          <BarChart data={transformedData}>
            <CartesianGrid strokeDasharray="3 3" />
            <XAxis dataKey="name" />
            <YAxis />
            <Tooltip />
            <Legend />
            {dataKeys.map((key: string, idx: number) => (
              <Bar
                key={key}
                dataKey={key}
                fill={COLORS[idx % COLORS.length]}
              />
            ))}
          </BarChart>
        )
    }
  }

  return (
    <div className="space-y-4">
      {hints.title && (
        <h3 className="text-lg font-semibold text-center text-primary">{hints.title}</h3>
      )}
      <ResponsiveContainer width="100%" height={400}>
        {renderChart()}
      </ResponsiveContainer>
    </div>
  )
}
