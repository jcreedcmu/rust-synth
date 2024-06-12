import { CSSProperties, useRef } from "react";
import { getScrollbarDims } from "./roll-util";

export type VScrollBarProps = {
  height: number,
  scrollTop: number,
  onScroll: (top: number) => void,
  content_height: number,
  x: number,
  y: number,
};

export function VScrollBar(props: VScrollBarProps): JSX.Element {
  // The reason for this fudge factor is that if the div is too narrow, the browser
  // won't even draw the vertical scrollbar. I found that in firefox, if I set the div
  // width to *exactly* the scroll bar width, it doesn't display. So we make the div a bit
  // wider and scoot it to the left by the same amount.
  const SCROLLBAR_EXTRA = 12;

  const ref = useRef<HTMLDivElement>(null);
  const s = getScrollbarDims();
  const { height, content_height, x, y, onScroll } = props;
  const style: CSSProperties = {
    height,
    left: x - SCROLLBAR_EXTRA,
    top: y,
    width: s.width + SCROLLBAR_EXTRA,
    overflowX: 'hidden',
    overflowY: 'scroll',
    position: 'absolute',
  };
  const c =
    <div ref={ref} style={style}
      onScroll={() => onScroll(ref.current!.scrollTop)}>
      <div style={{ height: content_height }}></div>
    </div>;
  return c;
}
