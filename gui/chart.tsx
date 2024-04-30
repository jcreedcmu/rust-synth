import * as Scale from '@visx/scale';
import {
  AnimatedAxis, // any of these can be non-animated equivalents
  AnimatedGrid,
  AnimatedLineSeries,
  XYChart,
  Tooltip,
  LineSeries,
  Grid,
  Axis,
} from '@visx/xychart';
import {
  Complex,
  add,
  complex,
  divide,
  exp,
  multiply
} from 'mathjs'

type Datum = { x: number, y: number }

function lg(x: number) {
  return Math.log(x) / Math.log(2);
}


const accessors = {
  xAccessor: (d: Datum) => d.x,
  yAccessor: (d: Datum) => d.y,
};



export function Chart(params: { lowp_param: number }): JSX.Element {
  const { lowp_param } = params;

  const data1: Datum[] = [];
  for (let i = 0; i <= 50; i++) {
    const a = i / 50;
    let freq = Math.pow(2, (1 - a) * lg(10) + a * lg(22050));

    const amp = (1 - lowp_param) * (divide(1, add(1, multiply(-lowp_param, exp(complex(0, -2 * Math.PI * freq / 44100))))) as Complex).toPolar().r;
    data1.push({ x: lg(freq), y: 20 * lg(amp) });
  }

  return <XYChart height={300}
    xScale={{ type: 'linear', domain: [lg(10), lg(22050)], zero: false }}
    yScale={{ type: 'linear', domain: [-50, 0] }}>
    <Axis orientation="bottom" tickFormat={x => Math.pow(2, x).toString().substr(0, 5)} />
    <Axis orientation="left" />
    <Grid numTicks={4} />
    <LineSeries dataKey="Filter" data={data1} {...accessors} />
    <Tooltip
      snapTooltipToDatumX
      snapTooltipToDatumY
      showVerticalCrosshair
      showSeriesGlyphs
      renderTooltip={({ tooltipData, colorScale }) => (
        <div>
          <div style={{ color: colorScale!(tooltipData!.nearestDatum!.key) }}>
            {tooltipData!.nearestDatum!.key}
          </div>
          {Math.pow(2, accessors.xAccessor(tooltipData!.nearestDatum!.datum as any))}
          {', '}
          {accessors.yAccessor(tooltipData!.nearestDatum!.datum as any)}
        </div>
      )}
    />
  </XYChart >;
}
