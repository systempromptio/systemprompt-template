import { renderChart as _renderChart } from './cc-charts-render.js';
import { initChartSetup } from './cc-charts-setup.js';

export { renderChart, updateHourlyCharts, update7dData, update30dData } from './cc-charts-render.js';

initChartSetup(_renderChart);
